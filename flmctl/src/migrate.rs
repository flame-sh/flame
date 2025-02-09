/*
Copyright 2024 The Flame Authors.
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

use std::error::Error;

use flame_rs::apis::FlameContext;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use url::Url;

pub async fn run(_: &FlameContext, url: &str, sql: &str) -> Result<(), Box<dyn Error>> {
    let uri = Url::parse(url)?;

    match uri.scheme() {
        "sqlite" => {
            Sqlite::database_exists(url).await?;
            Sqlite::create_database(url).await?;

            let db = SqlitePool::connect(url).await?;
            let migrations = std::path::Path::new(&sql);
            let migrator = sqlx::migrate::Migrator::new(migrations).await?;
            migrator.run(&db).await?;

            Ok(())
        }

        _ => Ok(()),
    }
}
