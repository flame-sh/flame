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

pub type MutexPtr<T> = Arc<std::sync::Mutex<T>>;
pub type AsyncPtr<T> = Arc<tokio::sync::Mutex<T>>;

pub fn new_ptr<T>(t: T) -> MutexPtr<T> {
    Arc::new(std::sync::Mutex::new(t))
}

pub fn new_async_ptr<T>(t: T) -> AsyncPtr<T> {
    Arc::new(tokio::sync::Mutex::new(t))
}

// pub type AsyncPtr<T> = Arc<tokio::sync::Mutex<T>>;

// #[derive(Clone, Debug)]
// pub struct CondPtr<T> {
//     pub ptr: MutexPtr<T>,
//     pub cond: Arc<Condvar>,
// }

// impl<T: Clone> CondPtr<T> {
//     pub fn new(value: T) -> Self {
//         CondPtr {
//             ptr: Arc::new(Mutex::new(value)),
//             cond: Arc::new(Condvar::new()),
//         }
//     }

//     pub fn wait_while<F>(&self, f: F) -> Result<MutexGuard<T>, FlameError>
//     where
//         F: Fn(&T) -> bool,
//     {
//         // TODO(k82cn): switch to condvar.wait_while when it works.
//         // let ptr = lock_ptr!(self.ptr)?;
//         // let _guard = self
//         //     .cond
//         //     .wait_while(ptr, f)
//         //     .map_err(|_| FlameError::Internal("condptr error".to_string()))?;

//         loop {
//             let ptr = lock_ptr!(self.ptr)?;
//             let cond = f(&*ptr);
//             if cond {
//                 return Ok(ptr);
//             }
//             let _gard = self
//                 .cond
//                 .wait(ptr)
//                 .map_err(|_| FlameError::Internal("condptr error".to_string()))?;
//         }
//     }

//     pub fn modify<F>(&self, mut mod_fn: F) -> Result<MutexGuard<T>, FlameError>
//     where
//         F: FnMut(&mut T) -> Result<(), FlameError>,
//     {
//         let mut ptr = lock_ptr!(self.ptr)?;
//         mod_fn(&mut *ptr)?;

//         self.cond.notify_all();

//         Ok(ptr)
//     }
// }
