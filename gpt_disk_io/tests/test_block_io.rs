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
use std::fs::{self, File, OpenOptions};

fn test_block_io_adapter() {
    let mut bio = BlockIoAdapter::new(123, BlockSize::BS_512);
    assert_eq!(bio.block_size(), BlockSize::BS_512);
    assert_eq!(bio.storage(), &123);
    assert_eq!(bio.storage_mut(), &mut 123);
    let data: u32 = bio.take_storage();
    assert_eq!(data, 123);
}

fn test_block_io_read<Io>(mut bio: Io) -> Io
where
    Io: BlockIo,
{
    let mut buf = vec![0; 512];

    // Read first block.
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

    bio
}

fn test_block_io_write1<Io>(mut bio: Io) -> Result<Io, Io::Error>
where
    Io: BlockIo,
{
    let mut buf = vec![0; 512];

    // Write first block.
    buf[0] = 5;
    buf[511] = 6;
    bio.write_blocks(Lba(0), &buf)?;

    // Write first block.
    buf[0] = 7;
    buf[511] = 8;
    bio.write_blocks(Lba(1), &buf)?;

    bio.flush()?;

    Ok(bio)
}

fn test_block_io_write2<Io>(mut bio: Io) -> Io
where
    Io: BlockIo,
{
    let mut buf = vec![0; 512 * 2];

    // Write two blocks at once, at an offset of one block.
    buf[0] = 9;
    buf[511] = 10;
    buf[512] = 11;
    buf[1023] = 12;
    bio.write_blocks(Lba(1), &buf).expect("write_blocks failed");

    bio.flush().expect("flush failed");

    bio
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

fn test_slice_block_io() {
    let mut data = vec![0; 512 * 3];

    // Write data to the beginning and end of the first two blocks.
    data[0] = 1;
    data[511] = 2;
    data[512] = 3;
    data[1023] = 4;

    test_block_io_read(BlockIoAdapter::new(data.as_slice(), BlockSize::BS_512));
    // Test that writes to a read-only slice fail.
    assert_eq!(
        test_block_io_write1(BlockIoAdapter::new(
            data.as_slice(),
            BlockSize::BS_512
        )),
        Err(SliceBlockIoError::ReadOnly)
    );

    let bio = BlockIoAdapter::new(data.as_mut_slice(), BlockSize::BS_512);
    test_block_io_read(bio);

    test_block_io_write1(BlockIoAdapter::new(
        data.as_mut_slice(),
        BlockSize::BS_512,
    ))
    .unwrap();
    assert_eq!(data[0], 5);
    assert_eq!(data[511], 6);
    assert_eq!(data[512], 7);
    assert_eq!(data[1023], 8);

    test_block_io_write2(BlockIoAdapter::new(
        data.as_mut_slice(),
        BlockSize::BS_512,
    ));
    assert_eq!(data[512], 9);
    assert_eq!(data[1023], 10);
    assert_eq!(data[1024], 11);
    assert_eq!(data[1535], 12);
}

#[cfg(feature = "alloc")]
fn test_vec_block_io() {
    let mut data = vec![0; 512 * 3];

    // Write data to the beginning and end of the first two blocks.
    data[0] = 1;
    data[511] = 2;
    data[512] = 3;
    data[1023] = 4;

    let mut bio = BlockIoAdapter::new(data.clone(), BlockSize::BS_512);
    assert_eq!(bio.num_blocks(), Ok(3));
    let bio = test_block_io_read(bio);

    let bio = test_block_io_write1(bio).unwrap();
    {
        let data = bio.storage();
        assert_eq!(data[0], 5);
        assert_eq!(data[511], 6);
        assert_eq!(data[512], 7);
        assert_eq!(data[1023], 8);
    }

    let bio = test_block_io_write2(bio);
    {
        let data = bio.storage();
        assert_eq!(data[512], 9);
        assert_eq!(data[1023], 10);
        assert_eq!(data[1024], 11);
        assert_eq!(data[1535], 12);
    }
}

#[cfg(feature = "std")]
fn test_std_block_io() {
    let path = "tmp_test_block_io_file.bin";
    let empty = vec![0; 512 * 3];

    {
        // Write data to the beginning and end of the first two blocks.
        let mut data = empty.clone();
        data[0] = 1;
        data[511] = 2;
        data[512] = 3;
        data[1023] = 4;

        fs::write(path, data).unwrap();
        let file = File::open(path).unwrap();

        test_block_io_read(BlockIoAdapter::new(file, BlockSize::BS_512));
        fs::remove_file(path).unwrap();
    };

    {
        fs::write(path, &empty).unwrap();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();

        test_block_io_write1(BlockIoAdapter::new(file, BlockSize::BS_512))
            .unwrap();

        let data = fs::read(path).unwrap();
        assert_eq!(data.len(), 512 * 3);
        assert_eq!(data[0], 5);
        assert_eq!(data[511], 6);
        assert_eq!(data[512], 7);
        assert_eq!(data[1023], 8);
        fs::remove_file(path).unwrap();
    }

    {
        fs::write(path, &empty).unwrap();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        test_block_io_write2(BlockIoAdapter::new(file, BlockSize::BS_512));

        let data = fs::read(path).unwrap();
        assert_eq!(data.len(), 512 * 3);
        assert_eq!(data[512], 9);
        assert_eq!(data[1023], 10);
        assert_eq!(data[1024], 11);
        assert_eq!(data[1535], 12);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn test_block_io() {
    test_block_io_adapter();

    test_slice_block_io();

    #[cfg(feature = "alloc")]
    test_vec_block_io();

    #[cfg(feature = "std")]
    test_std_block_io();
}
