// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod common;

use common::check_derives;
use gpt_disk_types::{Crc32, U32Le};

#[test]
fn test_crc32_display() {
    check_derives::<Crc32>();

    let crc = Crc32(U32Le([0x12, 0x34, 0x56, 0x78]));
    assert_eq!(format!("{crc:#x}"), "0x78563412");
    assert_eq!(format!("{crc}"), "0x78563412");
}
