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

use std::sync::Arc;

use crate::controller::ControllerPtr;
use crate::model::{ExecutorInfoPtr, SessionInfoPtr, SnapShotPtr};
use crate::scheduler::actions::{
    ActionPtr, AllocateAction, BackfillAction, DispatchAction, ShuffleAction,
};
use crate::scheduler::allocator::{AllocatorPtr, Allocator};
use crate::scheduler::dispatcher::{DispatcherPtr, Dispatcher};

use common::FlameError;

const DEFAULT_SCHEDULE_INTERVAL: u64 = 500;

pub struct Context {
    pub snapshot: SnapShotPtr,
    // pub controller: ControllerPtr,
    pub actions: Vec<ActionPtr>,
    // pub plugins: PluginManagerPtr,
    pub dispatcher: DispatcherPtr,
    pub allocator: AllocatorPtr,
    pub schedule_interval: u64,
}

impl Context {
    pub fn new(controller: ControllerPtr) -> Result<Self, FlameError> {
        let snapshot = controller.snapshot()?;
        // let plugins = PluginManager::setup(&snapshot.clone())?;
        let dispatcher = Arc::new(Dispatcher::new(snapshot.clone(), controller.clone())?);
        let allocator = Arc::new(Allocator::new(snapshot.clone(), controller.clone())?);

        Ok(Context {
            snapshot,
            // plugins,
            // controller,
            dispatcher,
            allocator,
            // TODO(k82cn): Add ActionManager for them.
            actions: vec![
                AllocateAction::new_ptr(),
                DispatchAction::new_ptr(),
                ShuffleAction::new_ptr(),
                BackfillAction::new_ptr(),
            ],
            schedule_interval: DEFAULT_SCHEDULE_INTERVAL,
        })
    }
}
