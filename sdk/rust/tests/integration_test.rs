/*
Copyright 2023 The Flame Authors.
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

use std::sync::{Arc, Mutex};

use futures::future::try_join_all;

use flame_rs as flame;

use flame::{
    apis::{FlameError, SessionState, TaskState},
    client::{SessionAttributes, Task, TaskInformer},
    lock_ptr, new_ptr,
};

const FLAME_DEFAULT_ADDR: &str = "http://127.0.0.1:30080";

const FLAME_DEFAULT_APP: &str = "flmtest";

pub struct DefaultTaskInformer {
    pub succeed: i32,
    pub failed: i32,
    pub error: i32,
}

impl TaskInformer for DefaultTaskInformer {
    fn on_update(&mut self, task: Task) {
        match task.state {
            TaskState::Succeed => self.succeed += 1,
            TaskState::Failed => self.failed += 1,
            _ => {}
        }
    }

    fn on_error(&mut self, _: FlameError) {
        self.error += 1;
    }
}

#[tokio::test]
async fn test_create_session() -> Result<(), FlameError> {
    let conn = flame::client::connect(FLAME_DEFAULT_ADDR).await?;

    let ssn_attr = SessionAttributes {
        application: FLAME_DEFAULT_APP.to_string(),
        slots: 1,
        common_data: None,
    };
    let ssn = conn.create_session(&ssn_attr).await?;

    assert_eq!(ssn.state, SessionState::Open);

    ssn.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_create_multiple_sessions() -> Result<(), FlameError> {
    let conn = flame::client::connect(FLAME_DEFAULT_ADDR).await?;

    let ssn_num = 10;

    for _ in 0..ssn_num {
        let ssn_attr = SessionAttributes {
            application: FLAME_DEFAULT_APP.to_string(),
            slots: 1,
            common_data: None,
        };
        let ssn = conn.create_session(&ssn_attr).await?;

        assert_eq!(ssn.state, SessionState::Open);

        ssn.close().await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_create_session_with_tasks() -> Result<(), FlameError> {
    let conn = flame::client::connect(FLAME_DEFAULT_ADDR).await?;

    let ssn_attr = SessionAttributes {
        application: FLAME_DEFAULT_APP.to_string(),
        slots: 1,
        common_data: None,
    };
    let ssn = conn.create_session(&ssn_attr).await?;

    assert_eq!(ssn.state, SessionState::Open);

    let informer = new_ptr!(DefaultTaskInformer {
        succeed: 0,
        failed: 0,
        error: 0,
    });

    let task_num = 100;
    let mut tasks = vec![];
    for _ in 0..task_num {
        let task = ssn.run_task(None, informer.clone());
        tasks.push(task);
    }

    try_join_all(tasks).await?;

    {
        let informer = lock_ptr!(informer)?;
        assert_eq!(informer.succeed, task_num);
    }

    ssn.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_create_multiple_sessions_with_tasks() -> Result<(), FlameError> {
    let conn = flame::client::connect(FLAME_DEFAULT_ADDR).await?;

    let ssn_attr = SessionAttributes {
        application: FLAME_DEFAULT_APP.to_string(),
        slots: 1,
        common_data: None,
    };
    let ssn_1 = conn.create_session(&ssn_attr).await?;
    assert_eq!(ssn_1.state, SessionState::Open);

    let ssn_2 = conn.create_session(&ssn_attr).await?;
    assert_eq!(ssn_2.state, SessionState::Open);

    let informer = new_ptr!(DefaultTaskInformer {
        succeed: 0,
        failed: 0,
        error: 0,
    });

    let task_num = 100;
    let mut tasks = vec![];

    for _ in 0..task_num {
        let task = ssn_1.run_task(None, informer.clone());
        tasks.push(task);
    }

    for _ in 0..task_num {
        let task = ssn_2.run_task(None, informer.clone());
        tasks.push(task);
    }

    try_join_all(tasks).await?;

    {
        let informer = lock_ptr!(informer)?;
        assert_eq!(informer.succeed, task_num * 2);
    }

    ssn_1.close().await?;
    ssn_2.close().await?;

    Ok(())
}
