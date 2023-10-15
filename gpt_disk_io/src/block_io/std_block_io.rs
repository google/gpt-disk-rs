// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{BlockIo, BlockIoAdapter};
use gpt_disk_types::{BlockSize, Lba};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Combination trait for types that impl [`Read`], [`Write`], and [`Seek`].
pub trait ReadWriteSeek: Read + Write + Seek {
    /// Get the number of blocks for the given `block_size`.
    ///
    /// The default implementation seeks to the end to get the number of
    /// bytes.
    fn num_blocks(&mut self, block_size: BlockSize) -> Result<u64, io::Error> {
        let block_size = block_size.to_u64();
        let num_bytes = self.seek(SeekFrom::End(0))?;
        Ok(num_bytes / block_size)
    }

    /// Read contiguous blocks.
    fn read_blocks(
        &mut self,
        block_size: BlockSize,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), io::Error> {
        block_size.assert_valid_block_buffer(dst);

        self.seek(SeekFrom::Start(start_lba.to_u64() * block_size.to_u64()))?;
        self.read_exact(dst)?;
        Ok(())
    }

    /// Write contiguous blocks.
    fn write_blocks(
        &mut self,
        block_size: BlockSize,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), io::Error> {
        block_size.assert_valid_block_buffer(src);

        self.seek(SeekFrom::Start(start_lba.to_u64() * block_size.to_u64()))?;
        self.write_all(src)?;
        Ok(())
    }
}

impl ReadWriteSeek for File {}
impl ReadWriteSeek for &File {}
impl<T> ReadWriteSeek for &mut T where T: Read + Write + Seek {}

impl<T> BlockIo for BlockIoAdapter<T>
where
    T: ReadWriteSeek,
{
    type Error = io::Error;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        self.storage.num_blocks(self.block_size)
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.storage.read_blocks(self.block_size, start_lba, dst)
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        self.storage.write_blocks(self.block_size, start_lba, src)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.storage.flush()
    }
}

impl BlockIo for BlockIoAdapter<&mut dyn ReadWriteSeek> {
    type Error = io::Error;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        self.storage.num_blocks(self.block_size)
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.storage.read_blocks(self.block_size, start_lba, dst)
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        self.storage.write_blocks(self.block_size, start_lba, src)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.storage.flush()
    }
}
