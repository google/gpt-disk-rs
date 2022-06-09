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

use anyhow::Result;
use common::check_derives;
use gpt_disk_io::{BlockIo, MutSliceBlockIo, SliceBlockIo, SliceBlockIoError};
use gpt_disk_types::{BlockSize, Lba};
#[cfg(feature = "std")]
use {gpt_disk_io::StdBlockIo, std::io::Cursor};

fn test_block_io_read<Io>(mut bio: Io) -> Result<(), Io::Error>
where
    Io: BlockIo,
{
    let mut buf = vec![0; 512];

    // Read first block.
    bio.read_blocks(Lba(0), &mut buf)?;
    assert_eq!(buf[0], 1);
    assert_eq!(buf[511], 2);

    // Read second block.
    bio.read_blocks(Lba(1), &mut buf)?;
    assert_eq!(buf[0], 3);
    assert_eq!(buf[511], 4);

    // Only three blocks.
    assert!(bio.read_blocks(Lba(3), &mut buf).is_err());

    // Read two blocks at once.
    let mut buf = vec![0; 1024];
    bio.read_blocks(Lba(0), &mut buf)?;
    assert_eq!(buf[0], 1);
    assert_eq!(buf[511], 2);
    assert_eq!(buf[512], 3);
    assert_eq!(buf[1023], 4);

    Ok(())
}

fn test_block_io_write1<Io>(mut bio: Io) -> Result<(), Io::Error>
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

    Ok(())
}

fn test_block_io_write2<Io>(mut bio: Io) -> Result<(), Io::Error>
where
    Io: BlockIo,
{
    let mut buf = vec![0; 512 * 2];

    // Write two blocks at once, at an offset of one block.
    buf[0] = 9;
    buf[511] = 10;
    buf[512] = 11;
    buf[1023] = 12;
    bio.write_blocks(Lba(1), &buf)?;

    bio.flush()?;

    Ok(())
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

fn test_slice_block_io() -> Result<()> {
    let mut data = vec![0; 512 * 3];

    // Write data to the beginning and end of the first two blocks.
    data[0] = 1;
    data[511] = 2;
    data[512] = 3;
    data[1023] = 4;

    test_block_io_read(SliceBlockIo::new(&mut data, BlockSize::BS_512))
        .unwrap();
    // Test that writes to a read-only slice fail.
    assert_eq!(
        test_block_io_write1(SliceBlockIo::new(&mut data, BlockSize::BS_512)),
        Err(SliceBlockIoError::ReadOnly)
    );

    let bio = MutSliceBlockIo::new(&mut data, BlockSize::BS_512);
    test_block_io_read(bio).unwrap();

    test_block_io_write1(MutSliceBlockIo::new(&mut data, BlockSize::BS_512))
        .unwrap();
    assert_eq!(data[0], 5);
    assert_eq!(data[511], 6);
    assert_eq!(data[512], 7);
    assert_eq!(data[1023], 8);

    test_block_io_write2(MutSliceBlockIo::new(&mut data, BlockSize::BS_512))
        .unwrap();
    assert_eq!(data[512], 9);
    assert_eq!(data[1023], 10);
    assert_eq!(data[1024], 11);
    assert_eq!(data[1535], 12);

    Ok(())
}

#[cfg(feature = "std")]
fn test_std_block_io() -> Result<()> {
    let empty = vec![0; 512 * 3];

    {
        // Write data to the beginning and end of the first two blocks.
        let mut data = empty.clone();
        data[0] = 1;
        data[511] = 2;
        data[512] = 3;
        data[1023] = 4;

        let mut cursor = Cursor::new(data);
        test_block_io_read(StdBlockIo::new(&mut cursor, BlockSize::BS_512))
            .unwrap();
    };

    {
        let mut cursor = Cursor::new(empty.clone());
        test_block_io_write1(StdBlockIo::new(&mut cursor, BlockSize::BS_512))
            .unwrap();
        let data = cursor.into_inner();
        assert_eq!(data.len(), 512 * 3);
        assert_eq!(data[0], 5);
        assert_eq!(data[511], 6);
        assert_eq!(data[512], 7);
        assert_eq!(data[1023], 8);
    }

    {
        let mut cursor = Cursor::new(empty.clone());
        test_block_io_write2(StdBlockIo::new(&mut cursor, BlockSize::BS_512))
            .unwrap();
        let data = cursor.into_inner();
        assert_eq!(data.len(), 512 * 3);
        assert_eq!(data[512], 9);
        assert_eq!(data[1023], 10);
        assert_eq!(data[1024], 11);
        assert_eq!(data[1535], 12);
    }

    Ok(())
}

#[test]
fn test_block_io() -> Result<()> {
    test_slice_block_io()?;

    #[cfg(feature = "std")]
    test_std_block_io()?;

    Ok(())
}
