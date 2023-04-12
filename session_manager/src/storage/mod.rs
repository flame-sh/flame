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
use std::sync::{Arc, Mutex};

use chrono::Utc;
use lazy_static::lazy_static;

use crate::model::{
    Executor, ExecutorID, FlameError, Session, SessionID, SessionStatus, Task, TaskID, TaskState,
};

mod engine;
mod util;

lazy_static! {
    static ref INSTANCE: Arc<Storage> = Arc::new(Storage {
        max_ssn_id: Mutex::new(0),
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
    engine: Option<Arc<dyn engine::Engine>>,
    sessions: Arc<Mutex<HashMap<SessionID, Arc<Mutex<Session>>>>>,
    executors: Arc<Mutex<HashMap<ExecutorID, Arc<Mutex<Executor>>>>>,
}

pub struct SnapShot {
    pub sessions: Vec<Session>,
    pub executors: Vec<Executor>,
}

impl Storage {
    pub fn snapshot(&self) -> Result<SnapShot, FlameError> {
        let mut res = SnapShot {
            sessions: vec![],
            executors: vec![],
        };

        {
            let ssn_map = self
                .sessions
                .lock()
                .map_err(|_| FlameError::Internal("session mutex".to_string()))?;

            for (_, ssn) in ssn_map.deref() {
                let ssn = ssn
                    .lock()
                    .map_err(|_| FlameError::Internal("session mutex".to_string()))?;
                res.sessions.push((*ssn).clone());
            }
        }

        {
            let exe_map = self
                .executors
                .lock()
                .map_err(|_| FlameError::Internal("executor mutex".to_string()))?;

            for (_, exe) in exe_map.deref() {
                let exe = exe
                    .lock()
                    .map_err(|_| FlameError::Internal("executor mutex".to_string()))?;
                res.executors.push((*exe).clone());
            }
        }

        Ok(res)
    }

    pub fn create_session(&self, app: String, slots: i32) -> Result<Session, FlameError> {
        let mut ssn_map = self
            .sessions
            .lock()
            .map_err(|_| FlameError::Internal("session mutex".to_string()))?;

        let mut ssn = Session::default();
        ssn.id = util::next_id(&self.max_ssn_id)?;
        ssn.slots = slots;
        ssn.application = app;
        ssn.creation_time = Utc::now();

        ssn_map.insert(ssn.id, Arc::new(Mutex::new(ssn.clone())));

        Ok(ssn)
    }

    pub fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn_map = self
            .sessions
            .lock()
            .map_err(|_| FlameError::Internal("session mutex".to_string()))?;

        let ssn = ssn_map.get(&id);
        match ssn {
            None => Err(FlameError::NotFound(id.to_string())),
            Some(s) => {
                let ssn = s
                    .lock()
                    .map_err(|_| FlameError::Internal("session mutex".to_string()))?;
                Ok(ssn.clone())
            }
        }
    }

    fn delete_session(&self, id: SessionID) -> Result<(), FlameError> {
        todo!()
    }

    fn update_session(&self, ssn: &Session) -> Result<Session, FlameError> {
        todo!()
    }

    pub fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut ssn_list = vec![];
        let ssn_map = self
            .sessions
            .lock()
            .map_err(|_| FlameError::Internal("session mutex".to_string()))?;

        for (_, ssn) in ssn_map.deref() {
            let ssn = ssn
                .lock()
                .map_err(|_| FlameError::Internal("session mutex".to_string()))?;
            ssn_list.push((*ssn).clone());
        }

        Ok(ssn_list)
    }

    pub(crate) fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<String>,
    ) -> Result<Task, FlameError> {
        let ssn_map = self
            .sessions
            .lock()
            .map_err(|_| FlameError::Internal("session mutex".to_string()))?;

        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let mut ssn = ssn
            .lock()
            .map_err(|_| FlameError::Internal("session mutex".to_string()))?;
        let task = Arc::new(Task {
            id: 0,
            ssn_id,
            input: task_input.clone(),
            output: None,
            creation_time: Default::default(),
            completion_time: None,
            state: TaskState::Pending,
        });

        ssn.tasks.push(task.clone());
        match ssn.tasks_index.get_mut(&task.state) {
            None => {
                ssn.tasks_index.insert(task.state, vec![task.clone()]);
            }
            Some(t) => {
                t.push(task.clone());
            }
        };

        Ok(task.deref().clone())
    }

    fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        todo!()
    }

    fn delete_task(&self, ssn_id: SessionID, id: TaskID) -> Result<(), FlameError> {
        todo!()
    }

    fn update_task(&self, t: &Task) -> Result<Task, FlameError> {
        todo!()
    }

    fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        todo!()
    }

    fn get_executor(&self, id: ExecutorID) -> Result<Executor, FlameError> {
        todo!()
    }

    fn unregister_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
        todo!()
    }

    fn update_executor(&self, e: &Executor) -> Result<Executor, FlameError> {
        todo!()
    }
}
