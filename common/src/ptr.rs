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


use crate::lock_ptr;
use std::sync::{Arc, Condvar, Mutex};

use crate::FlameError;

pub type MutexPtr<T> = Arc<Mutex<T>>;

#[derive(Clone, Debug)]
pub struct CondPtr<T> {
    pub ptr: MutexPtr<T>,
    pub cond: Arc<Condvar>,
}

impl<T> CondPtr<T> {
    pub fn wait_while<'a, F>(&self, condition: F) -> Result<(), FlameError>
    where
        F: FnMut(&mut T) -> bool,
    {
        let ptr = lock_ptr!(self.ptr)?;
        let _guard = self
            .cond
            .wait_while(ptr, condition)
            .map_err(|_| FlameError::Internal("condptr error".to_string()))?;

        Ok(())
    }

    // pub fn modify<'a, F>(&self, cond: F) -> Result<(), FlameError>
    // where F: FnMut(&MutexPtr<T>) -> Result<(), FlameError>,
    // {
    //     cond.call_once((&self.ptr, ))?;
    //     self.cond.notify_all();
    //
    //     Ok(())
    // }
}

impl<T> From<T> for CondPtr<T> {
    fn from(value: T) -> Self {
        CondPtr {
            ptr: Arc::new(Mutex::new(value)),
            cond: Arc::new(Condvar::new()),
        }
    }
}



