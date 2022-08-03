// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use anyhow::Result;
use common::{
    create_partition_entry, create_primary_header, create_secondary_header,
};
#[cfg(feature = "std")]
use gpt_disk_io::StdBlockIo;
use gpt_disk_io::{BlockIo, Disk, DiskError, MutSliceBlockIo, SliceBlockIo};
use gpt_disk_types::{BlockSize, GptPartitionEntryArray};
use std::io::{Cursor, Read};

fn load_test_disk() -> Vec<u8> {
    // Test data generated as follows:
    //
    // truncate --size 4MiB disk.bin
    // sgdisk disk.bin \
    //   --disk-guid=57a7feb6-8cd5-4922-b7bd-c78b0914e870 \
    //   --new=1:2048:4096 \
    //   --change-name='1:hello world!' \
    //   --partition-guid=1:37c75ffd-8932-467a-9c56-8cf1f0456b12 \
    //   --typecode=1:ccf0994f-f7e0-4e26-a011-843e38aa2eac
    // bzip2 disk.bin
    // mv disk.bin.bz2 gpt_disk_io/tests/
    let compressed_data = Cursor::new(include_bytes!("disk.bin.bz2"));

    let mut reader = bzip2_rs::DecoderReader::new(compressed_data);
    let mut disk = Vec::new();
    reader.read_to_end(&mut disk).unwrap();
    disk
}

fn test_disk_read<Io>(block_io: Io) -> Result<(), DiskError<Io::Error>>
where
    Io: BlockIo,
{
    let bs = BlockSize::BS_512;
    let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
    let mut disk = Disk::new(block_io)?;

    let primary_header = disk.read_primary_gpt_header(&mut block_buf)?;
    assert_eq!(primary_header, create_primary_header());

    let secondary_header = disk.read_secondary_gpt_header(&mut block_buf)?;
    assert_eq!(secondary_header, create_secondary_header());

    let expected_partition_entry = create_partition_entry();

    let check_partition_entry_array = |disk: &mut Disk<Io>, layout| {
        // First use the iter interface.
        {
            let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
            let mut iter = disk
                .gpt_partition_entry_array_iter(layout, &mut block_buf)
                .unwrap();
            let entry = iter.next().unwrap().unwrap();
            assert_eq!(entry, expected_partition_entry);
            assert!(entry.is_used());

            let entry = iter.next().unwrap().unwrap();
            assert!(!entry.is_used());
        }

        // Then check the whole array.
        let mut array_buf = vec![0u8; bs.to_usize().unwrap() * 34];
        let array = disk
            .read_gpt_partition_entry_array(layout, &mut array_buf)
            .unwrap();
        let entry = *array.get_partition_entry(0).unwrap();
        assert_eq!(entry, expected_partition_entry);
        assert!(entry.is_used());

        let entry = *array.get_partition_entry(1).unwrap();
        assert!(!entry.is_used());
    };

    // Check the primary partition entry array.
    check_partition_entry_array(
        &mut disk,
        primary_header.get_partition_entry_array_layout().unwrap(),
    );

    // Check the secondary partition entry array.
    check_partition_entry_array(
        &mut disk,
        secondary_header.get_partition_entry_array_layout().unwrap(),
    );

    Ok(())
}

fn test_disk_write<Io>(block_io: Io) -> Result<(), DiskError<Io::Error>>
where
    Io: BlockIo,
{
    let bs = BlockSize::BS_512;
    let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
    let mut disk = Disk::new(block_io)?;

    let primary_header = create_primary_header();
    let secondary_header = create_secondary_header();
    let partition_entry = create_partition_entry();

    disk.write_protective_mbr(&mut block_buf)?;
    disk.write_primary_gpt_header(&primary_header, &mut block_buf)?;
    disk.write_secondary_gpt_header(&secondary_header, &mut block_buf)?;

    let layout = primary_header.get_partition_entry_array_layout().unwrap();
    let mut bytes =
        vec![0; layout.num_bytes_rounded_to_block_as_usize(bs).unwrap()];
    let mut entry_array =
        GptPartitionEntryArray::new(layout, bs, &mut bytes).unwrap();
    *entry_array.get_partition_entry_mut(0).unwrap() = partition_entry;
    disk.write_gpt_partition_entry_array(&entry_array)?;

    entry_array.set_start_lba(secondary_header.partition_entry_lba.into());
    disk.write_gpt_partition_entry_array(&entry_array)?;

    disk.flush()?;

    Ok(())
}

fn test_with_slice(test_disk: &[u8]) {
    test_disk_read(SliceBlockIo::new(test_disk, BlockSize::BS_512)).unwrap();
}

fn test_with_mut_slice(test_disk: &[u8]) {
    let mut contents = test_disk.to_vec();

    // Test read.
    test_disk_read(MutSliceBlockIo::new(&mut contents, BlockSize::BS_512))
        .unwrap();

    // Test write.
    let mut new_contents = vec![0; contents.len()];
    test_disk_write(MutSliceBlockIo::new(&mut new_contents, BlockSize::BS_512))
        .unwrap();
    assert_eq!(contents, new_contents);
}

#[cfg(feature = "std")]
fn test_with_filelike(test_disk: &[u8]) -> Result<()> {
    let mut test_disk_cursor = Cursor::new(test_disk.to_vec());

    // Test read.
    test_disk_read(StdBlockIo::new(&mut test_disk_cursor, BlockSize::BS_512))?;

    // Test write.
    let mut new_disk = vec![0; 4 * 1024 * 1024];
    let mut new_disk_cursor = Cursor::new(&mut new_disk);
    test_disk_write(StdBlockIo::new(&mut new_disk_cursor, BlockSize::BS_512))?;
    assert_eq!(new_disk, test_disk);

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_disk() -> Result<()> {
    let test_disk = load_test_disk();

    test_with_slice(&test_disk);
    test_with_mut_slice(&test_disk);

    #[cfg(feature = "std")]
    test_with_filelike(&test_disk)?;

    Ok(())
}
