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

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use rusqlite::{Connection, Result};

use crate::FlameError;
use common::apis::{Session, SessionID, Task, TaskID};
use common::lock_ptr;
use common::ptr::{self, MutexPtr};

use crate::storage::engine::{Engine, EnginePtr};

pub struct SqliteEngine {
    conn: MutexPtr<Connection>,
}

const SSN_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    id              INTEGER AUTOINCREMENT PRIMARY KEY,
    application     TEXT NOT NULL,
    slots           INTEGER NOT NULL,

    common_data     BLOB,

    creation_time   REAL NOT NULL,
    completion_time REAL,

    state           INTEGER NOT NULL
)"#;

const TASK_TABLE_NAME: &str = "ssn__tasks";

const TASK_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS  (
    id              INTEGER AUTOINCREMENT PRIMARY KEY,
    ssn_id          INTEGER NOT NULL,

    input           BLOB,
    output          BLOB,

    creation_time   REAL NOT NULL,
    completion_time REAL,

    state           INTEGER NOT NULL
)"#;

impl SqliteEngine {
    pub fn new_ptr() -> Result<EnginePtr, FlameError> {
        let conn = Connection::open("flame.db").map_err(|e| FlameError::Internal(e.to_string()))?;

        conn.execute(SSN_TABLE, [])
            .map_err(|e| FlameError::Internal(e.to_string()))?;

        Ok(Arc::new(SqliteEngine {
            conn: ptr::new_ptr(conn),
        }))
    }
}

fn task_table(ssn_id: String) -> String {
    let mut task_table_name = String::from(TASK_TABLE_NAME);
    task_table_name.insert_str(4, &ssn_id);
    let mut task_table = String::from(TASK_TABLE);
    task_table.insert_str(27, &task_table_name);

    task_table
}

#[async_trait]
impl Engine for SqliteEngine {
    async fn persist_session(&self, ssn: &Session) -> Result<(), FlameError> {
        let mut conn = lock_ptr!(self.conn)?;
        let tx = conn
            .transaction()
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let common_data: Option<Vec<u8>> = ssn.common_data.clone().map(Bytes::into);

        tx.execute(
            r#"INSERT INTO sessions (application, slots, common_data, creation_time, state)
                   values (?1, ?2, ?3, ?4, ?5)"#,
            (
                &ssn.application,
                &ssn.slots,
                &common_data,
                &ssn.creation_time.to_string(),
                &(ssn.status.state as i32),
            ),
        )
        .map_err(|e| FlameError::Storage(e.to_string()))?;

        let ssn_id: String = tx.last_insert_rowid().to_string();
        tx.execute(&task_table(ssn_id), [])
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        Ok(())
    }
    async fn get_session(&self, _id: SessionID) -> Result<Session, FlameError> {
        todo!()
    }
    async fn delete_session(&self, _id: SessionID) -> Result<(), FlameError> {
        todo!()
    }
    async fn update_session(&self, _ssn: &Session) -> Result<(), FlameError> {
        todo!()
    }
    async fn find_session(&self) -> Result<Vec<Session>, FlameError> {
        // let mut conn = lock_ptr!(self.conn)?;
        // let mut stmt = conn.prepare("SELECT id, application, slots, common_data, creation_time, state, completion_time FROM sessions")?;
        // let ssns = stmt.query_map([], |row| {
        //     Ok(Session {
        //         id: row.get(0)?,
        //         name: row.get(1)?,
        //         age: row.get(2)?,
        //         data: row.get(3)?,
        //     })
        // })?;
        // .map_err(|e| FlameError::Storage(e.to_string()))?;

        todo!()
    }

    async fn persist_task(&self, _task: &Task) -> Result<(), FlameError> {
        todo!()
    }
    async fn get_task(&self, _ssn_id: SessionID, _id: TaskID) -> Result<Task, FlameError> {
        todo!()
    }
    async fn delete_task(&self, _ssn_id: SessionID, _id: TaskID) -> Result<(), FlameError> {
        todo!()
    }
    async fn update_task(&self, _t: &Task) -> Result<(), FlameError> {
        todo!()
    }
}
