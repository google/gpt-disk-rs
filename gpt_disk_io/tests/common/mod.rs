// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::fmt::{Debug, Display};
use core::hash::Hash;
use gpt_disk_types::{
    guid, Crc32, GptHeader, GptPartitionEntry, GptPartitionType, LbaLe, U32Le,
};
use std::collections::hash_map::DefaultHasher;

#[allow(dead_code)]
pub fn check_derives<T>()
where
    T: Clone
        + Copy
        + Debug
        + Default
        + Display
        + Eq
        + PartialEq
        + Hash
        + Ord
        + PartialOrd,
{
    let a = T::default();

    // PartialEq
    assert_eq!(a, a);

    // Clone / Copy
    assert_eq!(a, a.clone());
    let c: T = a;
    assert_eq!(a, c);

    // PartialOrd
    assert!(a >= a);

    // Debug/Display
    assert!(!format!("{a:?}").is_empty());
    format!("{a}");

    // Hash
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
}

pub fn create_primary_header() -> GptHeader {
    GptHeader {
        header_crc32: Crc32(U32Le::from_u32(0xa4877843)),
        my_lba: LbaLe::from_u64(1),
        alternate_lba: LbaLe::from_u64(8191),
        first_usable_lba: LbaLe::from_u64(34),
        last_usable_lba: LbaLe::from_u64(8158),
        disk_guid: guid!("57a7feb6-8cd5-4922-b7bd-c78b0914e870"),
        partition_entry_lba: LbaLe::from_u64(2),
        number_of_partition_entries: U32Le::from_u32(128),
        partition_entry_array_crc32: Crc32(U32Le::from_u32(0x9206adff)),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn create_secondary_header() -> GptHeader {
    GptHeader {
        header_crc32: Crc32(U32Le::from_u32(0xdbeb4c13)),
        my_lba: LbaLe::from_u64(8191),
        alternate_lba: LbaLe::from_u64(1),
        partition_entry_lba: LbaLe::from_u64(8159),
        ..create_primary_header()
    }
}

#[allow(dead_code)]
pub fn create_partition_entry() -> GptPartitionEntry {
    GptPartitionEntry {
        partition_type_guid: GptPartitionType(guid!(
            "ccf0994f-f7e0-4e26-a011-843e38aa2eac"
        )),
        unique_partition_guid: guid!("37c75ffd-8932-467a-9c56-8cf1f0456b12"),
        starting_lba: LbaLe::from_u64(2048),
        ending_lba: LbaLe::from_u64(4096),
        attributes: Default::default(),
        name: "hello world!".parse().unwrap(),
    }
}
