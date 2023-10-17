// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use common::check_derives;
use gpt_disk_io::{BlockIo, BlockIoAdapter, SliceBlockIoError};
use gpt_disk_types::{BlockSize, Lba};

#[cfg(feature = "std")]
use {
    gpt_disk_io::ReadWriteSeek,
    std::fs::{self, OpenOptions},
    std::io::Write,
};

#[test]
fn test_block_io_adapter() {
    let mut bio = BlockIoAdapter::new(123, BlockSize::BS_512);
    assert_eq!(bio.block_size(), BlockSize::BS_512);
    assert_eq!(bio.storage(), &123);
    assert_eq!(bio.storage_mut(), &mut 123);
    let data: u32 = bio.take_storage();
    assert_eq!(data, 123);
}

#[test]
fn test_slice_block_io_error() {
    check_derives::<SliceBlockIoError>();

    assert_eq!(
        SliceBlockIoError::Overflow.to_string(),
        "numeric overflow occurred"
    );
    assert_eq!(
        SliceBlockIoError::ReadOnly.to_string(),
        "attempted to write to a read-only byte slice"
    );
    assert_eq!(
        SliceBlockIoError::OutOfBounds {
            start_lba: Lba(1),
            length_in_bytes: 2
        }
        .to_string(),
        "out of bounds: start_lba=1, length_in_bytes=2",
    );
}

fn get_read_data() -> Vec<u8> {
    let mut data = vec![0; 512 * 3];

    // Write data to the beginning and end of the first two blocks.
    data[0] = 1;
    data[511] = 2;
    data[512] = 3;
    data[1023] = 4;

    data
}

fn check_read<S>(storage: S) -> S
where
    BlockIoAdapter<S>: BlockIo,
{
    let mut bio = BlockIoAdapter::new(storage, BlockSize::BS_512);
    assert_eq!(bio.num_blocks().unwrap(), 3);
    assert_eq!(BlockIo::block_size(&bio), BlockSize::BS_512);

    // Read first block.
    let mut buf = vec![0; 512];
    bio.read_blocks(Lba(0), &mut buf)
        .expect("read_blocks failed");
    assert_eq!(buf[0], 1);
    assert_eq!(buf[511], 2);

    // Read second block.
    bio.read_blocks(Lba(1), &mut buf)
        .expect("read_blocks failed");
    assert_eq!(buf[0], 3);
    assert_eq!(buf[511], 4);

    // Only three blocks.
    assert!(bio.read_blocks(Lba(3), &mut buf).is_err());

    // Read two blocks at once.
    let mut buf = vec![0; 1024];
    bio.read_blocks(Lba(0), &mut buf)
        .expect("read_blocks failed");
    assert_eq!(buf[0], 1);
    assert_eq!(buf[511], 2);
    assert_eq!(buf[512], 3);
    assert_eq!(buf[1023], 4);

    bio.take_storage()
}

fn check_write<S, G>(storage: S, get_bytes: G)
where
    BlockIoAdapter<S>: BlockIo,
    G: Fn(&BlockIoAdapter<S>) -> Vec<u8>,
{
    let mut bio = BlockIoAdapter::new(storage, BlockSize::BS_512);
    assert_eq!(bio.num_blocks().unwrap(), 3);

    let mut buf = vec![0; 512];

    // Write first block.
    buf[0] = 5;
    buf[511] = 6;
    bio.write_blocks(Lba(0), &buf).unwrap();

    // Write second block.
    buf[0] = 7;
    buf[511] = 8;
    bio.write_blocks(Lba(1), &buf).unwrap();
    bio.flush().unwrap();

    // Check write output.
    let mut expected = vec![0; 512 * 3];
    expected[0] = 5;
    expected[511] = 6;
    expected[512] = 7;
    expected[1023] = 8;
    assert_eq!(get_bytes(&bio), expected);

    // Write two blocks at once, at an offset of one block.
    let mut buf = vec![0; 1024];
    buf[0] = 9;
    buf[511] = 10;
    buf[512] = 11;
    buf[1023] = 12;
    bio.write_blocks(Lba(1), &buf).unwrap();
    bio.flush().unwrap();

    // Check write output.
    expected[512] = 9;
    expected[1023] = 10;
    expected[1024] = 11;
    expected[1535] = 12;
    assert_eq!(get_bytes(&bio), expected);
}

fn check_read_and_write<S, G>(storage: S, get_bytes: G)
where
    BlockIoAdapter<S>: BlockIo,
    G: Fn(&BlockIoAdapter<S>) -> Vec<u8>,
{
    let storage = check_read(storage);
    check_write(storage, get_bytes);
}

#[test]
fn test_block_io_slice_read() {
    let data = get_read_data();
    let storage: &[u8] = &data;

    check_read(storage);
}

#[test]
#[should_panic]
fn test_block_io_slice_write() {
    let data = get_read_data();
    let storage: &[u8] = &data;

    check_write(storage, |bio| bio.storage().to_vec());
}

#[test]
fn test_block_io_mut_slice() {
    let mut data = get_read_data();
    let storage: &mut [u8] = &mut data;

    check_read_and_write(storage, |bio| bio.storage().to_vec());
}

#[cfg(feature = "alloc")]
#[test]
fn test_block_io_vec() {
    let storage: Vec<u8> = get_read_data();
    check_read_and_write(storage, |bio| bio.storage().to_vec());
}

#[cfg(feature = "std")]
#[test]
fn test_block_io_file() {
    let path = "/tmp/test_block_io_std_1.bin";
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    file.write_all(&get_read_data()).unwrap();

    check_read_and_write(file, |_| fs::read(path).unwrap());

    fs::remove_file(path).unwrap();
}

#[cfg(feature = "std")]
#[test]
fn test_block_io_dyn_readwriteseek() {
    let path = "/tmp/test_block_io_std_2.bin";
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    file.write_all(&get_read_data()).unwrap();

    let storage: &mut dyn ReadWriteSeek = &mut file;
    check_read_and_write(storage, |_| fs::read(path).unwrap());

    fs::remove_file(path).unwrap();
}
