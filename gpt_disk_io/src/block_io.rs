// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub(crate) mod slice_block_io;

#[cfg(feature = "std")]
pub(crate) mod std_block_io;

use core::fmt::{Debug, Display};
use gpt_disk_types::{BlockSize, Lba};

/// Trait for reading from and writing to a block device.
pub trait BlockIo {
    /// IO error type.
    type Error: Debug + Display + Send + Sync + 'static;

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
    /// be a multiple of [`block_size`]. Implementations are permitted
    /// to panic if this precondition is not met, e.g. by calling
    /// [`BlockSize::assert_valid_block_buffer`].
    ///
    /// [`block_size`]: Self::block_size
    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error>;

    /// Write contiguous block to the disk. The `src` buffer size must
    /// be a multiple of [`block_size`]. Implementations are permitted
    /// to panic if this precondition is not met, e.g. by calling
    /// [`BlockSize::assert_valid_block_buffer`].
    ///
    /// Writes are not guaranteed to be complete until [`flush`] is
    /// called.
    ///
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

/// Adapter for types that can act as storage, but don't have a block
/// size. This is used to provide `BlockIo` impls for byte slices,
/// files, and various other types.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockIoAdapter<T> {
    storage: T,
    block_size: BlockSize,
}

impl<T> BlockIoAdapter<T> {
    /// Create a new `BlockIoAdapter`.
    #[must_use]
    pub fn new(storage: T, block_size: BlockSize) -> Self {
        Self {
            storage,
            block_size,
        }
    }

    /// Get the [`BlockSize`].
    #[must_use]
    pub fn block_size(&self) -> BlockSize {
        self.block_size
    }

    /// Get a reference to the underlying storage.
    #[must_use]
    pub fn storage(&self) -> &T {
        &self.storage
    }

    /// Get a mutable reference to the underlying storage.
    #[must_use]
    pub fn storage_mut(&mut self) -> &mut T {
        &mut self.storage
    }

    /// Consume the adapter and return the underlying storage.
    #[must_use]
    pub fn take_storage(self) -> T {
        self.storage
    }
}
