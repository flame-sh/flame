/*
Copyright 2023 The xflops Authors.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use lazy_static::lazy_static;

use crate::model;
use crate::model::{
    Executor, ExecutorID, ExecutorInfo, ExecutorPtr, Session, SessionID, SessionInfo, SessionPtr,
    SessionState, Task, TaskID, TaskPtr, TaskState,
};

use common::{lock_cond_ptr, lock_ptr};
use common::{trace::TraceFn, trace_fn, FlameError};

mod engine;
mod states;

lazy_static! {
    static ref INSTANCE: Arc<Storage> = Arc::new(Storage {
        max_ssn_id: Mutex::new(0),
        max_task_ids: Arc::new(Mutex::new(HashMap::new())),
        engine: None,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        executors: Arc::new(Mutex::new(HashMap::new())),
    });
}

pub fn instance() -> Arc<Storage> {
    Arc::clone(&INSTANCE)
}

pub struct Storage {
    max_ssn_id: Mutex<i64>,
    max_task_ids: Arc<Mutex<HashMap<SessionID, Mutex<i64>>>>,
    engine: Option<Arc<dyn engine::Engine>>,
    sessions: Arc<Mutex<HashMap<SessionID, SessionPtr>>>,
    executors: Arc<Mutex<HashMap<ExecutorID, ExecutorPtr>>>,
}

impl Storage {
    fn next_ssn_id(&self) -> Result<i64, FlameError> {
        let mut id = lock_ptr!(self.max_ssn_id)?;
        *id = *id + 1;

        Ok(*id.deref())
    }

    fn next_task_id(&self, ssn_id: &SessionID) -> Result<i64, FlameError> {
        let mut id_list = lock_ptr!(self.max_task_ids)?;
        if !id_list.contains_key(ssn_id) {
            id_list.insert(*ssn_id, Mutex::new(0));
        }

        let id = id_list.get(ssn_id).unwrap();
        let mut id = lock_ptr!(id)?;
        *id = *id + 1;

        Ok(*id.deref())
    }

    pub fn snapshot(&self) -> Result<model::SnapShot, FlameError> {
        let mut res = model::SnapShot {
            sessions: vec![],
            ssn_index: HashMap::new(),
            ssn_state_index: HashMap::new(),

            executors: vec![],
            exec_index: HashMap::new(),
            exec_state_index: HashMap::new(),
        };

        {
            let ssn_map = lock_ptr!(self.sessions)?;
            for (_, ssn) in ssn_map.deref() {
                let ssn = lock_cond_ptr!(ssn)?;
                let info = SessionInfo::from(&(*ssn));
                res.sessions.push(Rc::new(info));
            }
        }

        {
            let exe_map = lock_ptr!(self.executors)?;
            for (_, exe) in exe_map.deref() {
                let exe = lock_cond_ptr!(exe)?;
                let info = ExecutorInfo::from(&(*exe).clone());
                res.executors.push(Rc::new(info));
            }
        }

        // Build index without related locks.
        for ssn in &res.sessions {
            res.ssn_index.insert(ssn.id.clone(), ssn.clone());
            if !res.ssn_state_index.contains_key(&ssn.state) {
                res.ssn_state_index.insert(ssn.state.clone(), Vec::new());
            }

            if let Some(si) = res.ssn_state_index.get_mut(&ssn.state) {
                si.push(ssn.clone());
            }
        }

        for exec in &res.executors {
            res.exec_index.insert(exec.id.clone(), exec.clone());
            if !res.exec_state_index.contains_key(&exec.state) {
                res.exec_state_index.insert(exec.state.clone(), Vec::new());
            }

            if let Some(ei) = res.exec_state_index.get_mut(&exec.state) {
                ei.push(exec.clone());
            }

            // Update session's status
            if let Some(ssn_id) = &exec.ssn_id {
                match res.ssn_index.get_mut(ssn_id) {
                    // ssn.executors.insert(exec.id.clone, exec.clone());
                    None => {
                        log::warn!(
                            "Failed to find Session <{}> for Executor <{}>",
                            ssn_id,
                            exec.id
                        );
                    }
                    Some(ssn) => {
                        if let Some(ssn) = Rc::get_mut(ssn) {
                            (*ssn).executors.insert(exec.id.clone(), exec.clone());
                        }
                    }
                }
            }
        }

        Ok(res)
    }

    pub fn create_session(&self, app: String, slots: i32) -> Result<Session, FlameError> {
        let mut ssn_map = lock_ptr!(self.sessions)?;

        let mut ssn = Session::default();
        ssn.id = self.next_ssn_id()?;
        ssn.slots = slots;
        ssn.application = app;
        ssn.creation_time = Utc::now();

        if let Some(_) = &self.engine {
            // TODO(k82cn): persist session.
        }

        ssn_map.insert(ssn.id, SessionPtr::new(ssn.clone()));

        Ok(ssn)
    }

    pub fn close_session(&self, id: SessionID) -> Result<(), FlameError> {
        let ssn_ptr = self.get_session_ptr(id)?;
        let mut ssn = lock_cond_ptr!(ssn_ptr)?;
        if let Some(running_task) = ssn.tasks_index.get(&TaskState::Running) {
            if running_task.len() > 0 {
                return Err(FlameError::InvalidState(format!(
                    "can not close session with {} running tasks",
                    running_task.len()
                )));
            }
        }

        ssn.status.state = SessionState::Closed;
        Ok(())
    }

    pub fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn_ptr = self.get_session_ptr(id)?;
        let ssn = lock_cond_ptr!(ssn_ptr)?;
        Ok(ssn.clone())
    }

    fn get_session_ptr(&self, id: SessionID) -> Result<SessionPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn = ssn_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(ssn.clone())
    }

    fn get_task_ptr(&self, ssn_id: SessionID, task_id: TaskID) -> Result<TaskPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn_ptr = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let ssn = lock_cond_ptr!(ssn_ptr)?;
        let task_ptr = ssn
            .tasks
            .get(&task_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        Ok(task_ptr.clone())
    }

    pub fn delete_session(&self, id: SessionID) -> Result<(), FlameError> {
        let mut ssn_map = lock_ptr!(self.sessions)?;
        if let Some(ssn_ptr) = ssn_map.get(&id) {
            {
                let ssn = lock_cond_ptr!(ssn_ptr)?;
                if ssn.is_closed() {
                    return Err(FlameError::InvalidState(
                        "can not delete an open session".to_string(),
                    ));
                }
            }

            ssn_map.remove(&id);
        }

        Ok(())
    }

    pub fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut ssn_list = vec![];
        let ssn_map = lock_ptr!(self.sessions)?;

        for (_, ssn) in ssn_map.deref() {
            let ssn = lock_cond_ptr!(ssn)?;
            ssn_list.push((*ssn).clone());
        }

        Ok(ssn_list)
    }

    pub fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<String>,
    ) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let mut ssn = lock_cond_ptr!(ssn)?;

        if ssn.is_closed() {
            return Err(FlameError::InvalidState("session was closed".to_string()));
        }

        let state = TaskState::Pending;
        let task_id = self.next_task_id(&ssn_id)?;

        let task = Task {
            id: task_id,
            ssn_id,
            input: task_input.clone(),
            output: None,
            creation_time: Utc::now(),
            completion_time: None,
            state,
        };

        ssn.add_task(&task);

        Ok(task)
    }

    pub fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;

        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let ssn = lock_cond_ptr!(ssn)?;
        let task = ssn
            .tasks
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;
        let task = lock_cond_ptr!(task)?;
        Ok(task.clone())
    }

    // pub fn update_task_state(&self, t: &Task) -> Result<Task, FlameError> {
    //     let ssn_map = lock_ptr!(self.sessions)?;
    //
    //     let ssn = ssn_map
    //         .get(&t.ssn_id)
    //         .ok_or(FlameError::NotFound(t.ssn_id.to_string()))?;
    //
    //     let ssn = lock_cond_ptr!(ssn)?;
    //     let task = ssn
    //         .tasks
    //         .get(&t.id)
    //         .ok_or(FlameError::NotFound(t.id.to_string()))?;
    //
    //     let mut task = lock_cond_ptr!(task)?;
    //     task.state = t.state;
    //
    //     Ok((*task).clone())
    // }

    // fn delete_task(&self, _ssn_id: SessionID, _id: TaskID) -> Result<(), FlameError> {
    //     todo!()
    // }

    pub fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        let mut exe_map = lock_ptr!(self.executors)?;
        let exe = ExecutorPtr::new(e.clone());
        exe_map.insert(e.id.clone(), exe);

        Ok(())
    }

    // pub fn unregister_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
    //     let mut exe_map = lock_ptr!(self.executors)?;
    //     exe_map.remove(&id);
    //
    //     Ok(())
    // }

    fn get_executor_ptr(&self, id: ExecutorID) -> Result<ExecutorPtr, FlameError> {
        let exe_map = lock_ptr!(self.executors)?;
        let exe = exe_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(exe.clone())
    }

    pub async fn wait_for_session(&self, id: ExecutorID) -> Result<Session, FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(exe_ptr)?;

        let ssn_id = (*state).wait_for_session().await?;
        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        let ssn = lock_cond_ptr!(ssn_ptr)?;

        Ok((*ssn).clone())
    }

    pub fn bind_session(&self, id: ExecutorID, ssn_id: SessionID) -> Result<(), FlameError> {
        trace_fn!("Storage::bind_session");

        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(exe_ptr)?;

        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        state.bind_session(ssn_ptr)?;

        Ok(())
    }

    pub fn bind_session_completed(&self, id: ExecutorID) -> Result<(), FlameError> {
        trace_fn!("Storage::bind_session_completed");

        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(exe_ptr)?;

        state.bind_session_completed()?;

        Ok(())
    }

    pub fn launch_task(&self, id: ExecutorID) -> Result<Option<Task>, FlameError> {
        trace_fn!("Storage::launch_task");
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(exe_ptr.clone())?;
        let (ssn_id, task_id) = {
            let exec = lock_cond_ptr!(exe_ptr)?;
            (exec.ssn_id.clone(), exec.task_id.clone())
        };
        let ssn_id = ssn_id.ok_or(FlameError::InvalidState(
            "no session in bound executor".to_string(),
        ))?;

        //
        if let Some(task_id) = task_id {
            log::warn!(
                "Re-launch the task <{}/{}>",
                ssn_id.clone(),
                task_id.clone()
            );
            let task_ptr = self.get_task_ptr(ssn_id, task_id)?;

            let task = lock_cond_ptr!(task_ptr)?;
            return Ok(Some((*task).clone()));
        }

        let ssn_ptr = self.get_session_ptr(ssn_id)?;
        return Ok(state.launch_task(ssn_ptr)?);
    }

    pub fn complete_task(&self, id: ExecutorID) -> Result<(), FlameError> {
        trace_fn!("Storage::complete_task");
        let exe_ptr = self.get_executor_ptr(id)?;
        let (ssn_id, task_id) = {
            let exe = lock_cond_ptr!(exe_ptr)?;
            (
                exe.ssn_id.clone().ok_or(FlameError::InvalidState(
                    "no session in executor".to_string(),
                ))?,
                exe.task_id
                    .clone()
                    .ok_or(FlameError::InvalidState("no task in executor".to_string()))?,
            )
        };

        let task_ptr = self.get_task_ptr(ssn_id, task_id)?;
        let ssn_ptr = self.get_session_ptr(ssn_id)?;

        let state = states::from(exe_ptr.clone())?;
        state.complete_task(ssn_ptr, task_ptr)?;

        Ok(())
    }

    pub fn unbind_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(exe_ptr.clone())?;
        state.unbind_executor()?;

        Ok(())
    }

    pub fn unbind_executor_completed(&self, id: ExecutorID) -> Result<(), FlameError> {
        let exe_ptr = self.get_executor_ptr(id)?;
        let state = states::from(exe_ptr.clone())?;

        state.unbind_executor_completed()?;

        Ok(())
    }
}
