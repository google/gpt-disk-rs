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

use gpt_disk_types::{GptHeader, GptPartitionEntry, Guid, MasterBootRecord};
use std::mem;

#[test]
fn test_layouts() {
    assert_eq!(mem::size_of::<Guid>(), 16);
    assert_eq!(mem::align_of::<Guid>(), 1);

    assert_eq!(mem::size_of::<GptHeader>(), 92);
    assert_eq!(mem::align_of::<GptHeader>(), 1);

    assert_eq!(mem::size_of::<GptPartitionEntry>(), 128);
    assert_eq!(mem::align_of::<GptPartitionEntry>(), 1);

    assert_eq!(mem::size_of::<MasterBootRecord>(), 512);
    assert_eq!(mem::align_of::<MasterBootRecord>(), 1);
}
