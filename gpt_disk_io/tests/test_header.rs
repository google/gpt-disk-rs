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

use common::{check_derives, create_primary_header};
use gpt_disk_types::{
    Crc32, GptHeader, GptHeaderRevision, GptHeaderSignature,
    GptPartitionEntryArrayLayout, GptPartitionEntrySize,
    GptPartitionEntrySizeError, Lba, U32Le,
};

#[test]
fn test_signature() {
    check_derives::<GptHeaderSignature>();

    assert_eq!(
        GptHeaderSignature::EFI_COMPATIBLE_PARTITION_TABLE_HEADER.to_u64(),
        0x5452415020494645
    );
}

#[test]
fn test_revision() {
    check_derives::<GptHeaderRevision>();
    assert_eq!(GptHeaderRevision::VERSION_1_0.0.to_u32(), 0x00010000);
    assert_eq!(GptHeaderRevision::VERSION_1_0.major(), 1);
    assert_eq!(GptHeaderRevision::VERSION_1_0.minor(), 0);

    let rev = GptHeaderRevision(U32Le::from_u32(0x1234_5678));
    assert_eq!(rev.major(), 0x1234);
    assert_eq!(rev.minor(), 0x5678);
}

#[test]
fn test_header_signature() {
    let header = create_primary_header();
    assert!(header.is_signature_valid());
}

#[test]
fn test_header_crc32() {
    let mut header = create_primary_header();
    assert_eq!(
        header.calculate_header_crc32(),
        Crc32(U32Le::from_u32(0xa4877843))
    );

    header.update_header_crc32();
    assert_eq!(header.header_crc32, Crc32(U32Le::from_u32(0xa4877843)));
}

#[test]
fn test_header_impls() {
    check_derives::<GptHeader>();

    let mut header = create_primary_header();

    assert_eq!(header.to_string(), "GptHeader { signature: Signature(\"EFI PART\"), revision: 0x00010000, header_size: 92, header_crc32: 0xa4877843, my_lba: 1, alternate_lba: 8191, first_usable_lba: 34, last_usable_lba: 8158, disk_guid: 57a7feb6-8cd5-4922-b7bd-c78b0914e870, partition_entry_lba: 2, number_of_partition_entries: 128, size_of_partition_entry: 128, partition_entry_array_crc32: 0x9206adff }");

    // Test invalid signature.
    header.signature.0 .0[0] = 0xef;
    assert!(header.to_string().starts_with(
        "GptHeader { signature: Signature(Invalid: 0x54524150204946ef),"
    ));
}

#[test]
fn test_partition_entry_size() {
    check_derives::<GptPartitionEntrySize>();
    check_derives::<GptPartitionEntrySizeError>();

    assert_eq!(GptPartitionEntrySize::new(128).unwrap().to_u32(), 128);
    assert_eq!(GptPartitionEntrySize::default().to_u32(), 128);
    assert!(GptPartitionEntrySize::new(0).is_err());
    assert!(GptPartitionEntrySize::new(64).is_err());
    assert!(GptPartitionEntrySize::new(130).is_err());
}

#[test]
fn test_header_partition_layout() {
    let mut header = create_primary_header();

    header.size_of_partition_entry = U32Le::from_u32(256);
    assert_eq!(
        header.get_partition_entry_array_layout().unwrap(),
        GptPartitionEntryArrayLayout {
            start_lba: Lba(2),
            entry_size: GptPartitionEntrySize::new(256).unwrap(),
            num_entries: 128
        }
    );

    header.size_of_partition_entry = U32Le::from_u32(64);
    assert!(header.get_partition_entry_array_layout().is_err());
}
