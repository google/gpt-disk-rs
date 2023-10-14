// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::fmt::{Debug, Display};
use gpt_disk_types::{BlockSize, Lba};

/// Trait for reading from and writing to a block device.
pub trait BlockIo {
    /// IO error type.
    type Error: Debug + Display + Send + Sync + 'static;

    /// Panic if the `buffer` size is not a multiple of [`block_size`].
    ///
    /// [`block_size`]: Self::block_size
    fn assert_valid_buffer(&self, buffer: &[u8]) {
        assert!(self.block_size().is_multiple_of_block_size(buffer.len()));
    }

    /// Get the [`BlockSize`]. The return value is not allowed to
    /// change.
    fn block_size(&self) -> BlockSize;

    /// Get the number of logical blocks in the disk.
    ///
    /// If the underlying storage has a number of bytes that are not
    /// evenly divisible by [`block_size`], the implementation should
    /// return the number of whole blocks. In that case, the partial
    /// block at the end will not be accessible.
    ///
    /// [`block_size`]: Self::block_size
    fn num_blocks(&mut self) -> Result<u64, Self::Error>;

    /// Read contiguous blocks from the disk. The `dst` buffer size must
    /// be a multiple of [`block_size`]. Implementations can use
    /// [`assert_valid_buffer`] to check this.
    ///
    /// [`assert_valid_buffer`]: Self::assert_valid_buffer
    /// [`block_size`]: Self::block_size
    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error>;

    /// Write contiguous block to the disk. The `src` buffer size must
    /// be a multiple of [`block_size`]. Implementations can use
    /// [`assert_valid_buffer`] to check this.
    ///
    /// Writes are not guaranteed to be complete until [`flush`] is
    /// called.
    ///
    /// [`assert_valid_buffer`]: Self::assert_valid_buffer
    /// [`block_size`]: Self::block_size
    /// [`flush`]: Self::flush
    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error>;

    /// Flush any pending writes to the device.
    fn flush(&mut self) -> Result<(), Self::Error>;
}
