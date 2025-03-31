// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{BlockIo, BlockIoAdapter};
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::Range;
use gpt_disk_types::{BlockSize, Lba};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Error type used for `&[u8]` and `&mut [u8]` versions of [`BlockIoAdapter`].
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SliceBlockIoError {
    /// Numeric overflow occurred.
    #[default]
    Overflow,

    /// Attempted to write to a read-only byte slice.
    ReadOnly,

    /// A read or write is out of bounds.
    OutOfBounds {
        /// Start LBA.
        start_lba: Lba,

        /// Length in bytes.
        length_in_bytes: usize,
    },
}

impl Display for SliceBlockIoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("numeric overflow occurred"),
            Self::ReadOnly => {
                f.write_str("attempted to write to a read-only byte slice")
            }
            Self::OutOfBounds {
                start_lba,
                length_in_bytes,
            } => {
                write!(
                    f,
                    "out of bounds: start_lba={start_lba}, length_in_bytes={length_in_bytes}"
                )
            }
        }
    }
}

impl core::error::Error for SliceBlockIoError {}

#[track_caller]
fn buffer_byte_range_opt(
    block_size: BlockSize,
    start_lba: Lba,
    buf: &[u8],
) -> Option<Range<usize>> {
    let start_lba = usize::try_from(start_lba).ok()?;
    let start_byte = start_lba.checked_mul(block_size.to_usize()?)?;
    let end_byte = start_byte.checked_add(buf.len())?;
    Some(start_byte..end_byte)
}

#[track_caller]
fn buffer_byte_range(
    block_size: BlockSize,
    start_lba: Lba,
    buf: &[u8],
) -> Result<Range<usize>, SliceBlockIoError> {
    buffer_byte_range_opt(block_size, start_lba, buf)
        .ok_or(SliceBlockIoError::Overflow)
}

#[track_caller]
fn num_blocks(
    storage: &[u8],
    block_size: BlockSize,
) -> Result<u64, SliceBlockIoError> {
    let storage_len = u64::try_from(storage.len())
        .map_err(|_| SliceBlockIoError::Overflow)?;

    Ok(storage_len / block_size.to_u64())
}

#[track_caller]
fn read_blocks(
    storage: &[u8],
    block_size: BlockSize,
    start_lba: Lba,
    dst: &mut [u8],
) -> Result<(), SliceBlockIoError> {
    block_size.assert_valid_block_buffer(dst);

    let src = storage
        .get(buffer_byte_range(block_size, start_lba, dst)?)
        .ok_or(SliceBlockIoError::OutOfBounds {
            start_lba,
            length_in_bytes: dst.len(),
        })?;
    dst.copy_from_slice(src);
    Ok(())
}

fn write_blocks(
    storage: &mut [u8],
    block_size: BlockSize,
    start_lba: Lba,
    src: &[u8],
) -> Result<(), SliceBlockIoError> {
    block_size.assert_valid_block_buffer(src);

    let dst = storage
        .get_mut(buffer_byte_range(block_size, start_lba, src)?)
        .ok_or(SliceBlockIoError::OutOfBounds {
            start_lba,
            length_in_bytes: src.len(),
        })?;
    dst.copy_from_slice(src);
    Ok(())
}

impl BlockIo for BlockIoAdapter<&[u8]> {
    type Error = SliceBlockIoError;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        num_blocks(self.storage, self.block_size)
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        read_blocks(self.storage, self.block_size, start_lba, dst)
    }

    fn write_blocks(
        &mut self,
        _start_lba: Lba,
        _src: &[u8],
    ) -> Result<(), Self::Error> {
        Err(Self::Error::ReadOnly)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl BlockIo for BlockIoAdapter<&mut [u8]> {
    type Error = SliceBlockIoError;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        num_blocks(self.storage, self.block_size)
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        read_blocks(self.storage, self.block_size, start_lba, dst)
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        write_blocks(self.storage, self.block_size, start_lba, src)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl BlockIo for BlockIoAdapter<Vec<u8>> {
    type Error = SliceBlockIoError;

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        num_blocks(&self.storage, self.block_size)
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        read_blocks(&self.storage, self.block_size, start_lba, dst)
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        write_blocks(&mut self.storage, self.block_size, start_lba, src)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
