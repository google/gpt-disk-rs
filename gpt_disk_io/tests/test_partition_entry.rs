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
use gpt_disk_types::{
    GptPartitionAttributes, GptPartitionEntry, GptPartitionName,
    GptPartitionType, Guid, U16Le, U64Le,
};

#[test]
fn test_partition_type() {
    check_derives::<GptPartitionType>();

    assert_eq!(GptPartitionType::UNUSED.to_string(), "UNUSED");

    let guid = Guid::new(
        0x01234567_u32.to_le_bytes(),
        0x89ab_u16.to_le_bytes(),
        0xcdef_u16.to_le_bytes(),
        0x01,
        0x23,
        [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    );
    assert_eq!(
        GptPartitionType(guid).to_string(),
        "01234567-89ab-cdef-0123-456789abcdef"
    );
}

#[test]
fn test_required_partition_attribute() {
    check_derives::<GptPartitionAttributes>();

    let bits = 0x0000_0000_0000_0001u64;
    let mut attr = GptPartitionAttributes(U64Le::from_u64(bits));

    assert!(attr.required_partition());

    attr.update_required_partition(false);
    assert!(!attr.required_partition());
    attr.update_required_partition(true);
    assert!(attr.required_partition());
}

#[test]
fn test_no_block_io_protocol_attribute() {
    let bits = 0x0000_0000_0000_0002u64;
    let mut attr = GptPartitionAttributes(U64Le::from_u64(bits));

    assert!(attr.no_block_io_protocol());

    attr.update_no_block_io_protocol(false);
    assert!(!attr.no_block_io_protocol());
    attr.update_no_block_io_protocol(true);
    assert!(attr.no_block_io_protocol());
}

#[test]
fn test_legacy_bios_bootable_attribute() {
    let bits = 0x0000_0000_0000_0004u64;
    let mut attr = GptPartitionAttributes(U64Le::from_u64(bits));

    assert!(attr.legacy_bios_bootable());

    attr.update_legacy_bios_bootable(false);
    assert!(!attr.legacy_bios_bootable());
    attr.update_legacy_bios_bootable(true);
    assert!(attr.legacy_bios_bootable());
}

#[test]
fn test_type_specific_attributes() {
    let bits = 0x1234_0000_0000_0000u64;
    let mut attr = GptPartitionAttributes(U64Le::from_u64(bits));

    assert_eq!(attr.type_specific_attributes().to_u16(), 0x1234);

    attr.update_type_specific_attributes(U16Le::from_u16(0xabcd));
    assert_eq!(attr.type_specific_attributes().to_u16(), 0xabcd);
}

#[test]
fn test_partition_attribute_display() {
    let mut attr = GptPartitionAttributes(U64Le::from_u64(0));
    assert_eq!(attr.to_string(), "(empty)");

    attr.update_required_partition(true);
    assert_eq!(attr.to_string(), "required_partition (1)");

    attr.update_required_partition(false);
    attr.update_no_block_io_protocol(true);
    assert_eq!(attr.to_string(), "no_block_io_protocol (2)");

    attr.update_no_block_io_protocol(false);
    attr.update_legacy_bios_bootable(true);
    assert_eq!(attr.to_string(), "legacy_bios_bootable (4)");

    attr.update_required_partition(true);
    assert_eq!(
        attr.to_string(),
        "required_partition (1), legacy_bios_bootable (4)"
    );

    attr.update_type_specific_attributes(U16Le::from_u16(0x1234));
    assert_eq!(
        attr.to_string(),
        "required_partition (1), legacy_bios_bootable (4), type_specific(0x1234)"
    );
}

#[test]
fn test_partition_name() {
    check_derives::<GptPartitionName>();

    let mut name = GptPartitionName::default();

    assert!(name.is_empty());

    // "abc"
    name.set_char(0, 'a').unwrap();
    name.set_char(1, 'b').unwrap();
    name.set_char(2, 'c').unwrap();

    // 0xd800 is an invalid character.
    name.0[6] = 0x00;
    name.0[7] = 0xd8;

    assert_eq!(name.to_string(), "abc�");
    assert!(!name.is_empty());

    // Test with no trailing null.
    for i in 0..name.0.len() {
        name.0[i] = if (i % 2) == 0 { b'a' } else { 0 };
    }
    assert_eq!(name.to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
}

#[test]
fn test_partition_entry() {
    check_derives::<GptPartitionEntry>();
}
