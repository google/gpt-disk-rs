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

use core::fmt::{Debug, Display};
use gpt_disk_types::{BlockSize, Lba};

/// Trait for reading from and writing to a block device.
pub trait BlockIo {
    /// IO error type.
    type Error: Debug + Display + Send + Sync + 'static;

    /// Panic if the `buffer` size is zero, or not a multiple of
    /// [`block_size`].
    ///
    /// [`block_size`]: Self::block_size
    fn assert_valid_buffer(&self, buffer: &[u8]) {
        let buf_len = u64::try_from(buffer.len()).unwrap();
        let block_size = self.block_size().to_u64();
        assert_eq!(buf_len % block_size, 0);
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
    /// be a non-zero multiple of [`block_size`]. Implementations can
    /// use [`assert_valid_buffer`] to check this.
    ///
    /// [`assert_valid_buffer`]: Self::assert_valid_buffer
    /// [`block_size`]: Self::block_size
    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error>;

    /// Write contiguous block to the disk. The `src` buffer size must
    /// be a non-zero multiple of [`block_size`]. Implementations can
    /// use [`assert_valid_buffer`] to check this.
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
