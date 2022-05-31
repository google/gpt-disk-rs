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
use gpt_disk_types::{BlockSize, Lba, LbaLe, LbaRangeInclusive};

#[test]
fn test_lba() {
    check_derives::<Lba>();
}

#[test]
fn test_lba_le() {
    check_derives::<LbaLe>();
}

#[test]
fn test_lba_range_inclusive() {
    check_derives::<LbaRangeInclusive>();

    let block_size = BlockSize::new(512).unwrap();

    // Invalid range.
    assert!(LbaRangeInclusive::new(Lba(2), Lba(1)).is_none());

    // Valid range.
    let range = LbaRangeInclusive::new(Lba(1), Lba(2)).unwrap();
    assert_eq!(range.start(), 1);
    assert_eq!(range.end(), 2);
    assert_eq!(
        range.to_byte_range(block_size).unwrap(),
        512..=512 + 512 + (512 - 1)
    );
    assert_eq!(range.to_string(), "1..=2");

    // Test conversion from byte range.

    // Valid.
    assert_eq!(
        range,
        LbaRangeInclusive::from_byte_range(
            512..=512 + 512 + (512 - 1),
            block_size
        )
        .unwrap()
    );
    // Start is not on a block boundary.
    assert!(LbaRangeInclusive::from_byte_range(
        514..=512 + 512 + (512 - 1),
        block_size
    )
    .is_none());
    // End is not on a block boundary.
    assert!(LbaRangeInclusive::from_byte_range(
        512..=512 + 512 + (512 - 2),
        block_size
    )
    .is_none());
}

#[test]
fn test_block_size() {
    check_derives::<BlockSize>();

    assert_eq!(BlockSize::new(512).unwrap().to_u64(), 512);
    assert!(BlockSize::new(0).is_none());
    assert!(BlockSize::new(511).is_none());

    assert_eq!(BlockSize::from_usize(512).unwrap().to_u64(), 512);
    assert!(BlockSize::from_usize(0).is_none());

    assert_eq!(BlockSize::default().to_u64(), 512);
}
