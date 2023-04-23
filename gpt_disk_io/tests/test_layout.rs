// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gpt_disk_types::{GptHeader, GptPartitionEntry, Guid, MasterBootRecord};
use std::mem;

#[test]
fn test_layouts() {
    assert_eq!(mem::size_of::<Guid>(), 16);
    assert_eq!(mem::align_of::<Guid>(), 4);

    assert_eq!(mem::size_of::<GptHeader>(), 92);
    assert_eq!(mem::align_of::<GptHeader>(), 1);

    assert_eq!(mem::size_of::<GptPartitionEntry>(), 128);
    assert_eq!(mem::align_of::<GptPartitionEntry>(), 1);

    assert_eq!(mem::size_of::<MasterBootRecord>(), 512);
    assert_eq!(mem::align_of::<MasterBootRecord>(), 1);
}
