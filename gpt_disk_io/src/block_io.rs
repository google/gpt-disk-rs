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
///
/// See also [`BlockIoAdapter`].
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
///
/// Note that `BlockIoAdapter<T>` can be constructed for any sized type
/// `T`, but not all types provide a `BlockIo` impl for
/// `BlockIoAdapter<T>`.
///
/// # For byte slices
///
/// ```
/// use gpt_disk_io::gpt_disk_types::{BlockSize, Lba};
/// use gpt_disk_io::{BlockIo, BlockIoAdapter, SliceBlockIoError};
///
/// let mut one_block = [0; 512];
///
/// // Construct `BlockIoAdapter` for an immutable byte slice:
/// let data: &[u8] = &[0; 1024];
/// let mut bio = BlockIoAdapter::new(data, BlockSize::BS_512);
/// // Demonstrate that reading succeeds:
/// assert!(bio.read_blocks(Lba(0), &mut one_block).is_ok());
/// // But writing fails since the storage is immutable:
/// assert!(bio.write_blocks(Lba(0), &one_block).is_err());
///
/// // Construct `BlockIoAdapter` for a mutable byte slice:
/// let data: &mut [u8] = &mut [0; 512];
/// let mut bio = BlockIoAdapter::new(data, BlockSize::BS_512);
/// // Now both reading and writing succeed:
/// assert!(bio.read_blocks(Lba(0), &mut one_block).is_ok());
/// assert!(bio.write_blocks(Lba(0), &one_block).is_ok());
/// ```
///
/// # With the `alloc` feature
///
/// Construct a `BlockIoAdapter` that owns a `Vec<u8>`:
///
/// ```
/// use gpt_disk_io::gpt_disk_types::BlockSize;
/// use gpt_disk_io::{BlockIo, BlockIoAdapter, SliceBlockIoError};
///
/// #[cfg(feature = "alloc")]
/// fn example_alloc() -> Result<(), SliceBlockIoError> {
///     let data: Vec<u8> = vec![0; 512];
///     let mut bio = BlockIoAdapter::new(data, BlockSize::BS_512);
///     assert_eq!(bio.num_blocks()?, 1);
///
///     Ok(())
/// }
/// ```
///
/// # With the `std` feature
///
/// Construct `BlockIoAdapter` from various file-like types:
///
/// ```
/// use gpt_disk_io::gpt_disk_types::BlockSize;
/// use gpt_disk_io::{BlockIo, BlockIoAdapter};
/// use std::fs::{self, File};
/// use std::io::{self, Cursor};
/// use std::path::Path;
///
/// #[cfg(feature = "std")]
/// fn example_std(path: &Path) -> Result<(), io::Error> {
///     // Construct a `BlockIoAdapter` that takes ownership of a file.
///     // This also works for any type that implements the `ReadWriteSeek` trait.
///     let file = File::open(path)?;
///     let mut bio = BlockIoAdapter::new(file, BlockSize::BS_512);
///     assert_eq!(bio.num_blocks()?, 1);
///
///     // Construct a `BlockIoAdapter` that borrows a file.
///     let file = File::open(path)?;
///     let mut bio = BlockIoAdapter::new(&file, BlockSize::BS_512);
///     assert_eq!(bio.num_blocks()?, 1);
///
///     // Construct a `BlockIoAdapter` from another type that
///     // implements `Read + Write + Seek`, but does not directly implement
///     // `ReadWriteSeek`. Due to trait constraints, this requires an `&mut`
///     // borrow; transferring ownership of `cursor` would fail to compile.
///     let mut cursor = Cursor::new(vec![0; 512]);
///     let mut bio = BlockIoAdapter::new(&mut cursor, BlockSize::BS_512);
///     assert_eq!(bio.num_blocks()?, 1);
///
///     Ok(())
/// }
/// ```
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
