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

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Duration, Utc};
use sqlx::{migrate::MigrateDatabase, types::Json, FromRow, Sqlite, SqlitePool};

use crate::FlameError;
use common::{
    apis::{
        Application, ApplicationAttributes, ApplicationID, ApplicationState, CommonData, Session,
        SessionID, SessionState, SessionStatus, Shim, Task, TaskGID, TaskID, TaskInput, TaskOutput,
        TaskState,
    },
    trace::TraceFn,
    trace_fn,
};

use crate::storage::engine::{Engine, EnginePtr};

const SQLITE_SQL: &str = "migrations/sqlite";

#[derive(Clone, FromRow, Debug)]
struct ApplicationDao {
    pub name: ApplicationID,
    pub url: Option<String>,
    pub command: Option<String>,
    pub arguments: Option<Json<Vec<String>>>,
    pub environments: Option<Json<HashMap<String, String>>>,

    pub max_instances: i32,
    pub delay_release: i64,

    pub shim: i32,
    pub creation_time: i64,
    pub state: i32,
}

#[derive(Clone, FromRow, Debug)]
struct SessionDao {
    pub id: SessionID,
    pub application: String,
    pub slots: i32,

    pub common_data: Option<Vec<u8>>,
    pub creation_time: i64,
    pub completion_time: Option<i64>,

    pub state: i32,
}

#[derive(Clone, FromRow, Debug)]
struct TaskDao {
    pub id: TaskID,
    pub ssn_id: SessionID,

    pub input: Option<Vec<u8>>,
    pub output: Option<Vec<u8>>,

    pub creation_time: i64,
    pub completion_time: Option<i64>,

    pub state: i32,
}

pub struct SqliteEngine {
    pool: SqlitePool,
}

impl SqliteEngine {
    pub async fn new_ptr(url: &str) -> Result<EnginePtr, FlameError> {
        if !Sqlite::database_exists(url).await.unwrap_or(false) {
            Sqlite::create_database(url)
                .await
                .map_err(|e| FlameError::Storage(e.to_string()))?;
        }

        let db = SqlitePool::connect(url)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let migrations = std::path::Path::new(&SQLITE_SQL);
        let migrator = sqlx::migrate::Migrator::new(migrations)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;
        migrator
            .run(&db)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        Ok(Arc::new(SqliteEngine { pool: db }))
    }
}

#[async_trait]
impl Engine for SqliteEngine {
    async fn register_application(
        &self,
        name: String,
        attr: ApplicationAttributes,
    ) -> Result<(), FlameError> {
        trace_fn!("Sqlite::register_application");

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(format!("failed to begin TX: {e}")))?;

        let sql = "INSERT INTO applications (name, shim, command, arguments, environments, max_instances, delay_release, creation_time, state) VALUES (?, ?, ?, ?, ?, ?, ?, strftime ('%s', 'now'), 0) RETURNING *";
        let app: ApplicationDao = sqlx::query_as(sql)
            .bind(name)
            .bind::<i32>(attr.shim.into())
            .bind(attr.command)
            .bind(Json(attr.arguments))
            .bind(Json(attr.environments))
            .bind(attr.max_instances)
            .bind(attr.delay_release.num_seconds())
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(format!("failed to execute SQL: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(format!("failed to commit TX: {e}")))?;

        Ok(())
    }

    async fn get_application(&self, id: ApplicationID) -> Result<Application, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = "SELECT * FROM applications WHERE name=?";
        let app: ApplicationDao = sqlx::query_as(sql)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        app.try_into()
    }

    async fn find_application(&self) -> Result<Vec<Application>, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = "SELECT * FROM applications";
        let app: Vec<ApplicationDao> = sqlx::query_as(sql)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        Ok(app
            .iter()
            .map(Application::try_from)
            .filter_map(Result::ok)
            .collect())
    }

    async fn create_session(
        &self,
        app: String,
        slots: i32,
        common_data: Option<CommonData>,
    ) -> Result<Session, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let common_data: Option<Vec<u8>> = common_data.map(Bytes::into);
        let sql = "INSERT INTO sessions (application, slots, common_data, creation_time, state) VALUES (?, ?, ?, ?, ?) RETURNING *";
        let ssn: SessionDao = sqlx::query_as(sql)
            .bind(app)
            .bind(slots)
            .bind(common_data)
            .bind(Utc::now().timestamp())
            .bind(SessionState::Open as i32)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        ssn.try_into()
    }

    async fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = "SELECT * FROM sessions WHERE id=?";
        let ssn: SessionDao = sqlx::query_as(sql)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        ssn.try_into()
    }

    async fn delete_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = "DELETE FROM sessions WHERE id=? AND state=? RETURNING *";
        let ssn: SessionDao = sqlx::query_as(sql)
            .bind(id)
            .bind(SessionState::Closed as i32)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        ssn.try_into()
    }

    async fn close_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = r#"UPDATE sessions 
            SET state=?, completion_time=?
            WHERE id=? AND (SELECT COUNT(*) FROM tasks WHERE ssn_id=? AND state NOT IN (?, ?))=0
            RETURNING *"#;
        let ssn: SessionDao = sqlx::query_as(sql)
            .bind(SessionState::Closed as i32)
            .bind(Utc::now().timestamp())
            .bind(id)
            .bind(id)
            .bind(TaskState::Failed as i32)
            .bind(TaskState::Succeed as i32)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        ssn.try_into()
    }

    async fn find_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = "SELECT * FROM sessions";
        let ssn: Vec<SessionDao> = sqlx::query_as(sql)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        Ok(ssn
            .iter()
            .map(Session::try_from)
            .filter_map(Result::ok)
            .collect())
    }

    async fn create_task(
        &self,
        ssn_id: SessionID,
        input: Option<TaskInput>,
    ) -> Result<Task, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let input: Option<Vec<u8>> = input.map(Bytes::into);
        let sql = r#"INSERT INTO tasks (id, ssn_id, input, creation_time, state)
            VALUES (
                COALESCE((SELECT MAX(id)+1 FROM tasks WHERE ssn_id=?), 1),
                (SELECT id FROM sessions WHERE id=? AND state=?),
                ?,
                ?,
                ?)
            RETURNING *"#;
        let task: TaskDao = sqlx::query_as(sql)
            .bind(ssn_id)
            .bind(ssn_id)
            .bind(SessionState::Open as i32)
            .bind(input)
            .bind(Utc::now().timestamp())
            .bind(TaskState::Pending as i32)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        task.try_into()
    }
    async fn get_task(&self, gid: TaskGID) -> Result<Task, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = r#"SELECT * FROM tasks WHERE id=? AND ssn_id=?"#;
        let task: TaskDao = sqlx::query_as(sql)
            .bind(gid.task_id)
            .bind(gid.ssn_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        task.try_into()
    }
    async fn delete_task(&self, gid: TaskGID) -> Result<Task, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = r#"DELETE tasks WHERE id=? AND ssn_id=? RETURNING *"#;
        let task: TaskDao = sqlx::query_as(sql)
            .bind(gid.task_id)
            .bind(gid.ssn_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        task.try_into()
    }

    async fn retry_task(&self, gid: TaskGID) -> Result<Task, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = r#"UPDATE tasks SET state=? WHERE id=? AND ssn_id=? RETURNING *"#;
        let task: TaskDao = sqlx::query_as(sql)
            .bind(TaskState::Pending as i32)
            .bind(gid.task_id)
            .bind(gid.ssn_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        task.try_into()
    }

    async fn update_task(
        &self,
        gid: TaskGID,
        state: TaskState,
        output: Option<TaskOutput>,
    ) -> Result<Task, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let completion_time = match state {
            TaskState::Failed | TaskState::Succeed => Some(Utc::now().timestamp()),
            _ => None,
        };
        let output: Option<Vec<u8>> = output.map(Bytes::into);
        let sql = r#"UPDATE tasks SET state=?, completion_time=?, output=? WHERE id=? AND ssn_id=? RETURNING *"#;
        let task: TaskDao = sqlx::query_as(sql)
            .bind::<i32>(state.into())
            .bind(completion_time)
            .bind(output)
            .bind(gid.task_id)
            .bind(gid.ssn_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        task.try_into()
    }

    async fn find_tasks(&self, ssn_id: SessionID) -> Result<Vec<Task>, FlameError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        let sql = "SELECT * FROM tasks WHERE ssn_id=?";
        let task_list: Vec<TaskDao> = sqlx::query_as(sql)
            .bind(ssn_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| FlameError::Storage(e.to_string()))?;

        Ok(task_list
            .iter()
            .map(Task::try_from)
            .filter_map(Result::ok)
            .collect())
    }
}

impl TryFrom<&SessionDao> for Session {
    type Error = FlameError;

    fn try_from(ssn: &SessionDao) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ssn.id,
            application: ssn.application.clone(),
            slots: ssn.slots,
            common_data: ssn.common_data.clone().map(Bytes::from),
            creation_time: DateTime::<Utc>::from_timestamp(ssn.creation_time, 0)
                .ok_or(FlameError::Storage("invalid creation time".to_string()))?,
            completion_time: ssn
                .completion_time
                .map(|t| {
                    DateTime::<Utc>::from_timestamp(t, 0)
                        .ok_or(FlameError::Storage("invalid completion time".to_string()))
                })
                .transpose()?,
            tasks: HashMap::new(),
            tasks_index: HashMap::new(),
            status: SessionStatus {
                state: ssn.state.try_into()?,
            },
        })
    }
}

impl TryFrom<SessionDao> for Session {
    type Error = FlameError;

    fn try_from(ssn: SessionDao) -> Result<Self, Self::Error> {
        Session::try_from(&ssn)
    }
}

impl TryFrom<&TaskDao> for Task {
    type Error = FlameError;

    fn try_from(task: &TaskDao) -> Result<Self, Self::Error> {
        Ok(Self {
            id: task.id,
            ssn_id: task.ssn_id,
            input: task.input.clone().map(Bytes::from),
            output: task.output.clone().map(Bytes::from),

            creation_time: DateTime::<Utc>::from_timestamp(task.creation_time, 0)
                .ok_or(FlameError::Storage("invalid creation time".to_string()))?,
            completion_time: task
                .completion_time
                .map(|t| {
                    DateTime::<Utc>::from_timestamp(t, 0)
                        .ok_or(FlameError::Storage("invalid completion time".to_string()))
                })
                .transpose()?,

            state: task.state.try_into()?,
        })
    }
}

impl TryFrom<TaskDao> for Task {
    type Error = FlameError;

    fn try_from(ssn: TaskDao) -> Result<Self, Self::Error> {
        Task::try_from(&ssn)
    }
}

impl TryFrom<&ApplicationDao> for Application {
    type Error = FlameError;

    fn try_from(app: &ApplicationDao) -> Result<Self, Self::Error> {
        log::debug!("Application Shim is {}", app.shim);

        Ok(Self {
            name: app.name.clone(),
            state: ApplicationState::try_from(app.state)?,
            shim: Shim::try_from(app.shim)
                .map_err(|_| FlameError::Internal("unknown shim".to_string()))?,
            creation_time: DateTime::<Utc>::from_timestamp(app.creation_time, 0)
                .ok_or(FlameError::Storage("invalid creation time".to_string()))?,
            url: app.url.clone(),
            command: app.command.clone(),
            arguments: app.arguments.clone().map(|args| args.0).unwrap_or_default(),
            environments: app
                .environments
                .clone()
                .map(|envs| envs.0)
                .unwrap_or_default(),
            // TODO: make application configurable for work_dir.
            working_directory: String::from("/tmp"),
            max_instances: app.max_instances,
            delay_release: Duration::seconds(app.delay_release),
        })
    }
}

impl TryFrom<ApplicationDao> for Application {
    type Error = FlameError;

    fn try_from(ssn: ApplicationDao) -> Result<Self, Self::Error> {
        Application::try_from(&ssn)
    }
}

#[cfg(test)]
mod tests {
    use common::apis::ApplicationState;

    use super::*;

    #[test]
    fn test_get_application() -> Result<(), FlameError> {
        let url = format!("sqlite:///tmp/flame_test_app_{}.db", Utc::now().timestamp());
        let storage = tokio_test::block_on(SqliteEngine::new_ptr(&url))?;
        let app_1 = tokio_test::block_on(storage.get_application("flmexec".to_string()))?;

        assert_eq!(app_1.name, "flmexec");
        assert_eq!(app_1.state, ApplicationState::Enabled);

        Ok(())
    }

    #[test]
    fn test_single_session() -> Result<(), FlameError> {
        let url = format!(
            "sqlite:///tmp/flame_test_single_session_{}.db",
            Utc::now().timestamp()
        );
        let storage = tokio_test::block_on(SqliteEngine::new_ptr(&url))?;
        let ssn_1 = tokio_test::block_on(storage.create_session("flmexec".to_string(), 1, None))?;

        assert_eq!(ssn_1.id, 1);
        assert_eq!(ssn_1.application, "flmexec");
        assert_eq!(ssn_1.status.state, SessionState::Open);

        let task_1_1 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_1.id, 1);

        let task_1_2 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_2.id, 2);

        let task_list = tokio_test::block_on(storage.find_tasks(ssn_1.id))?;
        assert_eq!(task_list.len(), 2);

        let task_1_1 =
            tokio_test::block_on(storage.update_task(task_1_1.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_1_1.state, TaskState::Succeed);

        let task_1_2 =
            tokio_test::block_on(storage.update_task(task_1_2.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_1_2.state, TaskState::Succeed);

        let ssn_1 = tokio_test::block_on(storage.close_session(1))?;
        assert_eq!(ssn_1.status.state, SessionState::Closed);

        Ok(())
    }

    #[test]
    fn test_multiple_session() -> Result<(), FlameError> {
        let url = format!(
            "sqlite:///tmp/flame_test_multiple_session_{}.db",
            Utc::now().timestamp()
        );
        let storage = tokio_test::block_on(SqliteEngine::new_ptr(&url))?;
        let ssn_1 = tokio_test::block_on(storage.create_session("flmexec".to_string(), 1, None))?;

        assert_eq!(ssn_1.id, 1);
        assert_eq!(ssn_1.application, "flmexec");
        assert_eq!(ssn_1.status.state, SessionState::Open);

        let task_1_1 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_1.id, 1);

        let task_1_2 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_2.id, 2);

        let task_1_1 =
            tokio_test::block_on(storage.update_task(task_1_1.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_1_1.state, TaskState::Succeed);

        let task_1_2 =
            tokio_test::block_on(storage.update_task(task_1_2.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_1_2.state, TaskState::Succeed);

        let ssn_2 = tokio_test::block_on(storage.create_session("flmlog".to_string(), 1, None))?;

        assert_eq!(ssn_2.id, 2);
        assert_eq!(ssn_2.application, "flmlog");
        assert_eq!(ssn_2.status.state, SessionState::Open);

        let task_2_1 = tokio_test::block_on(storage.create_task(ssn_2.id, None))?;
        assert_eq!(task_2_1.id, 1);

        let task_2_2 = tokio_test::block_on(storage.create_task(ssn_2.id, None))?;
        assert_eq!(task_2_2.id, 2);

        let task_2_1 =
            tokio_test::block_on(storage.update_task(task_2_1.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_2_1.state, TaskState::Succeed);

        let task_2_2 =
            tokio_test::block_on(storage.update_task(task_2_2.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_2_2.state, TaskState::Succeed);

        let ssn_list = tokio_test::block_on(storage.find_session())?;
        assert_eq!(ssn_list.len(), 2);

        let ssn_1 = tokio_test::block_on(storage.close_session(1))?;
        assert_eq!(ssn_1.status.state, SessionState::Closed);
        let ssn_2 = tokio_test::block_on(storage.close_session(2))?;
        assert_eq!(ssn_2.status.state, SessionState::Closed);

        Ok(())
    }

    #[test]
    fn test_close_session_with_open_tasks() -> Result<(), FlameError> {
        let url = format!(
            "sqlite:///tmp/flame_test_close_session_with_open_tasks_{}.db",
            Utc::now().timestamp()
        );
        let storage = tokio_test::block_on(SqliteEngine::new_ptr(&url))?;
        let ssn_1 = tokio_test::block_on(storage.create_session("flmexec".to_string(), 1, None))?;

        assert_eq!(ssn_1.id, 1);
        assert_eq!(ssn_1.application, "flmexec");
        assert_eq!(ssn_1.status.state, SessionState::Open);

        let task_1_1 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_1.id, 1);

        let task_1_2 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_2.id, 2);

        let res = tokio_test::block_on(storage.close_session(1));
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    fn test_create_task_for_close_session() -> Result<(), FlameError> {
        let url = format!(
            "sqlite:///tmp/flame_test_create_task_for_close_session_{}.db",
            Utc::now().timestamp()
        );

        let storage = tokio_test::block_on(SqliteEngine::new_ptr(&url))?;
        let ssn_1 = tokio_test::block_on(storage.create_session("flmexec".to_string(), 1, None))?;

        assert_eq!(ssn_1.id, 1);
        assert_eq!(ssn_1.application, "flmexec");
        assert_eq!(ssn_1.status.state, SessionState::Open);

        let task_1_1 = tokio_test::block_on(storage.create_task(ssn_1.id, None))?;
        assert_eq!(task_1_1.id, 1);

        let task_1_1 =
            tokio_test::block_on(storage.update_task(task_1_1.gid(), TaskState::Succeed, None))?;
        assert_eq!(task_1_1.state, TaskState::Succeed);

        let ssn_1 = tokio_test::block_on(storage.close_session(1))?;
        assert_eq!(ssn_1.status.state, SessionState::Closed);

        let res = tokio_test::block_on(storage.create_task(ssn_1.id, None));
        assert!(res.is_err());

        Ok(())
    }
}
