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

mod allocate;
mod assign;
mod preempt;

pub use allocate::AllocateAction;
pub use assign::AssignAction;
pub use preempt::PreemptAction;

use crate::storage::SnapShot;
use crate::FlameError;

pub trait Action {
    fn execute(&self, snapshot: &mut SnapShot) -> Result<(), FlameError>;
}
