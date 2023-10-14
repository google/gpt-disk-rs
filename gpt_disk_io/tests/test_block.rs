// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use common::check_derives;
use gpt_disk_types::{BlockSize, Lba, LbaLe, LbaRangeInclusive, U64Le};

#[test]
fn test_lba() {
    check_derives::<Lba>();
}

#[test]
fn test_lba_le() {
    check_derives::<LbaLe>();

    assert_eq!(LbaLe::from(Lba(123)), LbaLe(U64Le::from_u64(123)));
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

    assert_eq!(BlockSize::default().to_u32(), 512);
    assert_eq!(BlockSize::default().to_u64(), 512);
}

#[test]
fn test_block_size_is_multiple() {
    assert!(BlockSize::BS_512.is_multiple_of_block_size(0));
    assert!(BlockSize::BS_512.is_multiple_of_block_size(512));
    assert!(BlockSize::BS_512.is_multiple_of_block_size(1024));

    assert!(!BlockSize::BS_512.is_multiple_of_block_size(1023));
    assert!(!BlockSize::BS_512.is_multiple_of_block_size(1025));
}

#[test]
#[should_panic]
fn test_block_size_is_multiple_panic() {
    let _ = BlockSize::BS_512.is_multiple_of_block_size(u128::MAX);
}
