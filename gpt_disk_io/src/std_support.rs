// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{BlockIo, DiskError, SliceBlockIoError};
use gpt_disk_types::{BlockSize, Lba};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Wrapper type that implements the [`BlockIo`] trait for a file-like
/// type that implements [`Read`], [`Write`], and [`Seek`].
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use gpt_disk_io::gpt_disk_types::BlockSize;
/// use gpt_disk_io::{Disk, StdBlockIo};
/// use std::fs::File;
///
/// let mut file = File::open("some/disk")?;
/// let block_io = StdBlockIo::new(&mut file, BlockSize::BS_512);
///
/// let mut block_buf = vec![0u8; 512];
/// let mut disk = Disk::new(block_io)?;
/// let header = disk.read_primary_gpt_header(&mut block_buf)?;
/// # Ok(())
/// # }
/// ```
pub struct StdBlockIo<'a, T>
where
    T: Read + Write + Seek,
{
    file: &'a mut T,
    block_size: BlockSize,
}

impl<'a, T> StdBlockIo<'a, T>
where
    T: Read + Write + Seek,
{
    /// Create an `StdBlockIo` from a file-like input. The input type must
    /// implement [`Read`], [`Write`], and [`Seek`].
    pub fn new(file: &'a mut T, block_size: BlockSize) -> Self {
        Self { file, block_size }
    }
}

impl<'a, T> BlockIo for StdBlockIo<'a, T>
where
    T: Read + Write + Seek,
{
    type Error = io::Error;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        let block_size = self.block_size().to_u64();
        let num_bytes = self.file.seek(SeekFrom::End(0))?;
        Ok(num_bytes / block_size)
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.assert_valid_buffer(dst);

        self.file.seek(SeekFrom::Start(
            start_lba.to_u64() * self.block_size().to_u64(),
        ))?;
        self.file.read_exact(dst)?;
        Ok(())
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        self.assert_valid_buffer(src);

        self.file.seek(SeekFrom::Start(
            start_lba.to_u64() * self.block_size().to_u64(),
        ))?;
        self.file.write_all(src)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.file.flush()
    }
}

impl<Custom> Error for DiskError<Custom> where Custom: Debug + Display {}

impl Error for SliceBlockIoError {}
