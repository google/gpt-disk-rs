// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use common::check_derives;
use gpt_disk_types::{
    Chs, DiskGeometry, Lba, MasterBootRecord, MbrPartitionRecord, U32Le,
};

#[test]
fn test_chs() {
    check_derives::<Chs>();

    assert_eq!(
        Chs::from_lba(Lba(8191), DiskGeometry::UNKNOWN)
            .unwrap()
            .as_tuple(),
        (0, 130, 2)
    );

    // Out of range errors.
    assert!(Chs::new(0xf000, 1, 1).is_none());
    assert!(Chs::new(1, 1, 0xf0).is_none());
}

#[test]
fn test_disk_geometry() {
    check_derives::<DiskGeometry>();
}

#[test]
fn test_mbr() {
    check_derives::<MasterBootRecord>();

    let mut mbr = MasterBootRecord {
        boot_strap_code: [0; 440],
        unique_mbr_disk_signature: [0x12, 0x34, 0x56, 0x78],
        unknown: [0x12, 0x34],
        partitions: [
            MbrPartitionRecord {
                boot_indicator: 0x12,
                start_chs: Chs::new(1, 2, 3).unwrap(),
                os_indicator: 0xab,
                end_chs: Chs::new(4, 5, 6).unwrap(),
                starting_lba: U32Le::from_u32(123),
                size_in_lba: U32Le::from_u32(456),
            },
            MbrPartitionRecord::default(),
            MbrPartitionRecord::default(),
            MbrPartitionRecord::default(),
        ],
        signature: [0x12, 0x34],
    };
    let expected = "MasterBootRecord {
boot_strap_code: [0; 440],
unique_mbr_disk_signature: 0x78563412,
unknown: 3412,
partitions: [MbrPartitionRecord {
boot_indicator: 0x12,
start_chs: CHS=1/2/3,
os_indicator: 0xab,
end_chs: CHS=4/5/6,
starting_lba: 123,
size_in_lba: 456 },
MbrPartitionRecord {
boot_indicator: 0x0,
start_chs: CHS=0/0/0,
os_indicator: 0x0,
end_chs: CHS=0/0/0,
starting_lba: 0,
size_in_lba: 0 },
MbrPartitionRecord {
boot_indicator: 0x0,
start_chs: CHS=0/0/0,
os_indicator: 0x0,
end_chs: CHS=0/0/0,
starting_lba: 0,
size_in_lba: 0 },
MbrPartitionRecord {
boot_indicator: 0x0,
start_chs: CHS=0/0/0,
os_indicator: 0x0,
end_chs: CHS=0/0/0,
starting_lba: 0,
size_in_lba: 0 }],
signature: 0x3412
}";
    assert_eq!(mbr.to_string(), expected.replace('\n', " "));

    mbr.boot_strap_code[0] = 1;
    assert!(mbr
        .to_string()
        .starts_with("MasterBootRecord { boot_strap_code: <non-zero>,"));
}
