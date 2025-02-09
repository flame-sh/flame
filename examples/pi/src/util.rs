/*
Copyright 2025 The Flame Authors.
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

use flame_rs::apis::FlameError;

#[allow(dead_code)]
pub fn zero_u32() -> Vec<u8> {
    vec![0u8, 0u8, 0u8, 0u8]
}

pub fn u32_to_bytes(i: u32) -> Vec<u8> {
    i.to_be_bytes().to_vec()
}

pub fn bytes_to_u32(v: Vec<u8>) -> Result<u32, FlameError> {
    if v.len() != 4 {
        return Err(FlameError::InvalidConfig(
            "Vec<u8> must contain exactly 4 bytes".to_string(),
        ));
    }

    let byte_array: [u8; 4] = v.try_into().unwrap();
    Ok(u32::from_be_bytes(byte_array))
}
