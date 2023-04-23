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

extern crate core;

use flame_client as flame;

use self::flame::{FlameError, Session, SessionAttributes, SessionState};

const FLAME_DEFAULT_ADDR: &str = "http://127.0.0.1:8080";

const FLAME_DEFAULT_APP: &str = "flmexec";

#[tokio::test]
async fn test_create_session() -> Result<(), FlameError> {
    flame::connect(FLAME_DEFAULT_ADDR).await?;

    let ssn_attr = SessionAttributes {
        application: FLAME_DEFAULT_APP.to_string(),
        slots: 1,
    };
    let ssn = Session::new(&ssn_attr).await?;

    assert_eq!(ssn.state, SessionState::Open);

    ssn.close().await?;

    Ok(())
}
