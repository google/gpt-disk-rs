// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use common::{
    create_partition_entry, create_primary_header, create_secondary_header,
};
#[cfg(feature = "std")]
use gpt_disk_io::StdBlockIo;
use gpt_disk_io::{BlockIo, Disk, MutSliceBlockIo, SliceBlockIo};
use gpt_disk_types::{BlockSize, GptPartitionEntryArray};
use std::io::{Cursor, Seek, SeekFrom, Write};

struct Data {
    off: u64,
    v: [u8; 16],
}

#[rustfmt::skip]
const SPARSE_DISK: &'static [Data] = &[
// Test data generated as follows:
//
// truncate --size 4MiB disk.bin
// sgdisk disk.bin \
//   --disk-guid=57a7feb6-8cd5-4922-b7bd-c78b0914e870 \
//   --new=1:2048:4096 \
//   --change-name='1:hello world!' \
//   --partition-guid=1:37c75ffd-8932-467a-9c56-8cf1f0456b12 \
//   --typecode=1:ccf0994f-f7e0-4e26-a011-843e38aa2eac
// hexdump -ve '"Data{off:0x%_ax,v:[" 16/1 "%u," "]},\n"' disk.bin \
//   | grep -v 'v:\[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,\]'
Data{off:0x1c0,v:[2,0,238,130,2,0,1,0,0,0,255,31,0,0,0,0,]},
Data{off:0x1f0,v:[0,0,0,0,0,0,0,0,0,0,0,0,0,0,85,170,]},
Data{off:0x200,v:[69,70,73,32,80,65,82,84,0,0,1,0,92,0,0,0,]},
Data{off:0x210,v:[67,120,135,164,0,0,0,0,1,0,0,0,0,0,0,0,]},
Data{off:0x220,v:[255,31,0,0,0,0,0,0,34,0,0,0,0,0,0,0,]},
Data{off:0x230,v:[222,31,0,0,0,0,0,0,182,254,167,87,213,140,34,73,]},
Data{off:0x240,v:[183,189,199,139,9,20,232,112,2,0,0,0,0,0,0,0,]},
Data{off:0x250,v:[128,0,0,0,128,0,0,0,255,173,6,146,0,0,0,0,]},
Data{off:0x400,v:[79,153,240,204,224,247,38,78,160,17,132,62,56,170,46,172,]},
Data{off:0x410,v:[253,95,199,55,50,137,122,70,156,86,140,241,240,69,107,18,]},
Data{off:0x420,v:[0,8,0,0,0,0,0,0,0,16,0,0,0,0,0,0,]},
Data{off:0x430,v:[0,0,0,0,0,0,0,0,104,0,101,0,108,0,108,0,]},
Data{off:0x440,v:[111,0,32,0,119,0,111,0,114,0,108,0,100,0,33,0,]},
Data{off:0x3fbe00,v:[79,153,240,204,224,247,38,78,160,17,132,62,56,170,46,172,]},
Data{off:0x3fbe10,v:[253,95,199,55,50,137,122,70,156,86,140,241,240,69,107,18,]},
Data{off:0x3fbe20,v:[0,8,0,0,0,0,0,0,0,16,0,0,0,0,0,0,]},
Data{off:0x3fbe30,v:[0,0,0,0,0,0,0,0,104,0,101,0,108,0,108,0,]},
Data{off:0x3fbe40,v:[111,0,32,0,119,0,111,0,114,0,108,0,100,0,33,0,]},
Data{off:0x3ffe00,v:[69,70,73,32,80,65,82,84,0,0,1,0,92,0,0,0,]},
Data{off:0x3ffe10,v:[19,76,235,219,0,0,0,0,255,31,0,0,0,0,0,0,]},
Data{off:0x3ffe20,v:[1,0,0,0,0,0,0,0,34,0,0,0,0,0,0,0,]},
Data{off:0x3ffe30,v:[222,31,0,0,0,0,0,0,182,254,167,87,213,140,34,73,]},
Data{off:0x3ffe40,v:[183,189,199,139,9,20,232,112,223,31,0,0,0,0,0,0,]},
Data{off:0x3ffe50,v:[128,0,0,0,128,0,0,0,255,173,6,146,0,0,0,0,]},
];

fn load_test_disk() -> Vec<u8> {
    let mut disk = vec![0; 4 * 1024 * 1024];
    let mut disk_cursor = Cursor::new(&mut disk);
    for i in SPARSE_DISK {
        disk_cursor
            .seek(SeekFrom::Start(i.off))
            .expect("seek failed");
        disk_cursor.write(&i.v).expect("write failed");
    }
    disk
}

fn test_disk_read<Io>(block_io: Io)
where
    Io: BlockIo,
{
    let bs = BlockSize::BS_512;
    let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
    let mut disk = Disk::new(block_io).unwrap();

    let primary_header = disk.read_primary_gpt_header(&mut block_buf).unwrap();
    assert_eq!(primary_header, create_primary_header());

    let secondary_header =
        disk.read_secondary_gpt_header(&mut block_buf).unwrap();
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
}

fn test_disk_write<Io>(block_io: Io)
where
    Io: BlockIo,
{
    let bs = BlockSize::BS_512;
    let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
    let mut disk = Disk::new(block_io).unwrap();

    let primary_header = create_primary_header();
    let secondary_header = create_secondary_header();
    let partition_entry = create_partition_entry();

    disk.write_protective_mbr(&mut block_buf).unwrap();
    disk.write_primary_gpt_header(&primary_header, &mut block_buf)
        .unwrap();
    disk.write_secondary_gpt_header(&secondary_header, &mut block_buf)
        .unwrap();

    let layout = primary_header.get_partition_entry_array_layout().unwrap();
    let mut bytes =
        vec![0; layout.num_bytes_rounded_to_block_as_usize(bs).unwrap()];
    let mut entry_array =
        GptPartitionEntryArray::new(layout, bs, &mut bytes).unwrap();
    *entry_array.get_partition_entry_mut(0).unwrap() = partition_entry;
    disk.write_gpt_partition_entry_array(&entry_array).unwrap();

    entry_array.set_start_lba(secondary_header.partition_entry_lba.into());
    disk.write_gpt_partition_entry_array(&entry_array).unwrap();

    disk.flush().unwrap();
}

fn test_with_slice(test_disk: &[u8]) {
    test_disk_read(SliceBlockIo::new(test_disk, BlockSize::BS_512));
}

fn test_with_mut_slice(test_disk: &[u8]) {
    let mut contents = test_disk.to_vec();

    // Test read.
    test_disk_read(MutSliceBlockIo::new(&mut contents, BlockSize::BS_512));

    // Test write.
    let mut new_contents = vec![0; contents.len()];
    test_disk_write(MutSliceBlockIo::new(&mut new_contents, BlockSize::BS_512));
    assert_eq!(contents, new_contents);
}

#[cfg(feature = "std")]
fn test_with_filelike(test_disk: &[u8]) {
    let mut test_disk_cursor = Cursor::new(test_disk.to_vec());

    // Test read.
    test_disk_read(StdBlockIo::new(&mut test_disk_cursor, BlockSize::BS_512));

    // Test write.
    let mut new_disk = vec![0; 4 * 1024 * 1024];
    let mut new_disk_cursor = Cursor::new(&mut new_disk);
    test_disk_write(StdBlockIo::new(&mut new_disk_cursor, BlockSize::BS_512));
    assert_eq!(new_disk, test_disk);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_disk() {
    let test_disk = load_test_disk();

    test_with_slice(&test_disk);
    test_with_mut_slice(&test_disk);

    #[cfg(feature = "std")]
    test_with_filelike(&test_disk);
}
