// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::BlockIo;
use bytemuck::{bytes_of, from_bytes};
use core::fmt::{self, Debug, Display, Formatter};
use core::mem;
use gpt_disk_types::{
    BlockSize, GptHeader, GptPartitionEntry, GptPartitionEntryArray,
    GptPartitionEntryArrayError, GptPartitionEntryArrayLayout, Lba,
    MasterBootRecord,
};

/// Iterator over entries in a partition entry array.
struct GptPartitionEntryIter<'disk, 'buf, Io: BlockIo> {
    disk: &'disk mut Disk<Io>,
    block_buf: &'buf mut [u8],
    layout: GptPartitionEntryArrayLayout,
    next_index: u32,
    current_lba: Lba,
    byte_offset_within_lba: usize,
    entry_size: usize,
}

impl<'disk, 'buf, Io: BlockIo> GptPartitionEntryIter<'disk, 'buf, Io> {
    fn new(
        disk: &'disk mut Disk<Io>,
        layout: GptPartitionEntryArrayLayout,
        block_buf: &'buf mut [u8],
    ) -> Result<Self, DiskError<Io::Error>> {
        let mut iter = Self {
            disk,
            block_buf,
            next_index: 0,
            current_lba: layout.start_lba,
            byte_offset_within_lba: 0,
            layout,
            entry_size: layout
                .entry_size
                .to_usize()
                .ok_or(DiskError::Overflow)?,
        };
        iter.set_current_lba(iter.current_lba)?;
        Ok(iter)
    }

    fn set_current_lba(
        &mut self,
        lba: Lba,
    ) -> Result<(), DiskError<Io::Error>> {
        self.current_lba = lba;
        self.byte_offset_within_lba = 0;
        Ok(self.disk.io.read_blocks(self.current_lba, self.block_buf)?)
    }

    fn read_current_entry(&mut self) -> Option<<Self as Iterator>::Item> {
        let entry_bytes = self.block_buf.get(
            self.byte_offset_within_lba
                ..self.byte_offset_within_lba + self.entry_size,
        )?;

        self.byte_offset_within_lba += self.entry_size;

        self.next_index += 1;

        Some(Ok(*from_bytes::<GptPartitionEntry>(
            &entry_bytes[..mem::size_of::<GptPartitionEntry>()],
        )))
    }
}

impl<'disk, 'buf, Io: BlockIo> Iterator
    for GptPartitionEntryIter<'disk, 'buf, Io>
{
    type Item = Result<GptPartitionEntry, DiskError<Io::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.layout.num_entries {
            return None;
        }

        if let Some(entry) = self.read_current_entry() {
            Some(entry)
        } else {
            let next_lba = Lba(self.current_lba.to_u64() + 1);
            if let Err(err) = self.set_current_lba(next_lba) {
                Some(Err(err))
            } else {
                self.read_current_entry()
            }
        }
    }
}

/// Workaround for using `impl Trait` with multiple lifetimes. See
/// <https://stackoverflow.com/a/50548538>.
pub trait Captures<'a, 'b> {}

impl<'a, 'b, T: ?Sized> Captures<'a, 'b> for T {}

/// Error type used by [`Disk`] methods.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum DiskError<IoError: Debug + Display> {
    /// The storage buffer is not large enough.
    BufferTooSmall,

    /// Numeric overflow occurred.
    Overflow,

    /// The partition entry size is larger than a single block.
    BlockSizeSmallerThanPartitionEntry,

    /// Error from a [`BlockIo`] implementation (see [`BlockIo::Error`]).
    ///
    /// [`BlockIo`]: crate::BlockIo
    /// [`BlockIo::Error`]: crate::BlockIo::Error
    Io(IoError),
}

impl<IoError> From<IoError> for DiskError<IoError>
where
    IoError: Debug + Display,
{
    fn from(err: IoError) -> Self {
        DiskError::Io(err)
    }
}

impl<IoError> Display for DiskError<IoError>
where
    IoError: Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall => f.write_str("storage buffer is too small"),
            Self::Overflow => f.write_str("numeric overflow occurred"),
            Self::BlockSizeSmallerThanPartitionEntry => {
                f.write_str("partition entries are larger than a single block")
            }
            Self::Io(io) => Display::fmt(io, f),
        }
    }
}

/// Read and write GPT disk data.
///
/// The disk is accessed via an object implementing the [`BlockIo`]
/// trait, so all reads and writes are on block boundaries. Writes are
/// not guaranteed to be completed until [`flush`] is called. This
/// happens automatically when the `Disk` is dropped, but if an error
/// occurs at that point it will be silently ignored so it is
/// recommended to call [`flush`] directly before dropping the disk.
///
/// Many of the methods on `Disk` take a `block_buf` argument, which is
/// a mutable byte buffer with a length of at least one block. (The
/// [`read_gpt_partition_entry_array`] and
/// [`write_gpt_partition_entry_array`] methods take a larger `storage`
/// argument that is multiple blocks in size.) These buffer arguments
/// allow `Disk` to avoid doing any internal memory allocation.
///
/// # Partition entry arrays
///
/// Partition entry arrays can be read in two ways: one block at a time
/// with [`gpt_partition_entry_array_iter`], or all at once with
/// [`read_gpt_partition_entry_array`]. The former allows a smaller
/// amount of memory usage bounded to the block size, while the latter
/// may be more efficient since all the blocks can be read at once.
///
/// Writing the array can currently only be done all at once via
/// [`write_gpt_partition_entry_array`]; a block-at-a-time method may be
/// added in the future.
///
/// [`flush`]: Self::flush
/// [`gpt_partition_entry_array_iter`]: Self::gpt_partition_entry_array_iter
/// [`read_gpt_partition_entry_array`]: Self::read_gpt_partition_entry_array
/// [`write_gpt_partition_entry_array`]: Self::write_gpt_partition_entry_array
pub struct Disk<Io: BlockIo> {
    io: Io,
}

impl<Io: BlockIo> Disk<Io> {
    /// Create a `Disk`.
    pub fn new(io: Io) -> Result<Self, DiskError<Io::Error>> {
        Ok(Self { io })
    }

    /// Clip the size of `block_buf` to a single block. Return
    /// `BufferTooSmall` if the buffer isn't big enough.
    fn clip_block_buf_size<'buf>(
        &self,
        block_buf: &'buf mut [u8],
    ) -> Result<&'buf mut [u8], DiskError<Io::Error>> {
        if let Some(block_size) = self.io.block_size().to_usize() {
            block_buf
                .get_mut(..block_size)
                .ok_or(DiskError::BufferTooSmall)
        } else {
            Err(DiskError::BufferTooSmall)
        }
    }

    /// Read the primary GPT header from the second block. No validation
    /// of the header is performed.
    pub fn read_primary_gpt_header(
        &mut self,
        block_buf: &mut [u8],
    ) -> Result<GptHeader, DiskError<Io::Error>> {
        self.read_gpt_header(Lba(1), block_buf)
    }

    /// Read the secondary GPT header from the last block. No validation
    /// of the header is performed.
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn read_secondary_gpt_header(
        &mut self,
        block_buf: &mut [u8],
    ) -> Result<GptHeader, DiskError<Io::Error>> {
        let num_blocks = self.io.num_blocks()?;
        let last_block =
            Lba(num_blocks.checked_sub(1).ok_or(DiskError::Overflow)?);
        self.read_gpt_header(last_block, block_buf)
    }

    /// Read a GPT header at the given [`Lba`]. No validation of the
    /// header is performed.
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn read_gpt_header(
        &mut self,
        lba: Lba,
        mut block_buf: &mut [u8],
    ) -> Result<GptHeader, DiskError<Io::Error>> {
        block_buf = self.clip_block_buf_size(block_buf)?;
        self.io.read_blocks(lba, block_buf)?;
        let bytes = block_buf
            .get(..mem::size_of::<GptHeader>())
            // OK to unwrap since the block size type guarantees a
            // minimum size greater than GptHeader.
            .unwrap();
        Ok(*from_bytes(bytes))
    }

    /// Read the entire partition entry array. The `storage` buffer must
    /// be at least [`layout.num_bytes_rounded_to_block`] in size.
    ///
    /// [`layout.num_bytes_rounded_to_block`]: GptPartitionEntryArrayLayout::num_bytes_rounded_to_block
    pub fn read_gpt_partition_entry_array<'buf>(
        &mut self,
        layout: GptPartitionEntryArrayLayout,
        storage: &'buf mut [u8],
    ) -> Result<GptPartitionEntryArray<'buf>, DiskError<Io::Error>> {
        let mut entry_array =
            GptPartitionEntryArray::new(layout, self.io.block_size(), storage)
                .map_err(|err| match err {
                    GptPartitionEntryArrayError::BufferTooSmall => {
                        DiskError::BufferTooSmall
                    }
                    GptPartitionEntryArrayError::Overflow => {
                        DiskError::Overflow
                    }
                })?;
        self.io
            .read_blocks(layout.start_lba, entry_array.storage_mut())?;
        Ok(entry_array)
    }

    /// Write an entire [`GptPartitionEntryArray`] to disk.
    pub fn write_gpt_partition_entry_array(
        &mut self,
        entry_array: &GptPartitionEntryArray,
    ) -> Result<(), DiskError<Io::Error>> {
        Ok(self.io.write_blocks(
            entry_array.layout().start_lba,
            entry_array.storage(),
        )?)
    }

    /// Get an iterator over partition entries. The `layout` parameter
    /// indicates where to read the entries from; see
    /// [`GptPartitionEntryArrayLayout`] for more.
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn gpt_partition_entry_array_iter<'disk, 'buf>(
        &'disk mut self,
        layout: GptPartitionEntryArrayLayout,
        mut block_buf: &'buf mut [u8],
    ) -> Result<
        impl Iterator<Item = Result<GptPartitionEntry, DiskError<Io::Error>>>
            + Captures<'disk, 'buf>,
        DiskError<Io::Error>,
    > {
        block_buf = self.clip_block_buf_size(block_buf)?;

        let entry_size =
            layout.entry_size.to_usize().ok_or(DiskError::Overflow)?;
        if entry_size > block_buf.len() {
            return Err(DiskError::BlockSizeSmallerThanPartitionEntry);
        }

        GptPartitionEntryIter::<'disk, 'buf>::new(self, layout, block_buf)
    }

    /// Write a protective MBR to the first block. If the block size is
    /// bigger than the MBR, the rest of the block will be filled with
    /// zeroes.
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn write_protective_mbr(
        &mut self,
        block_buf: &mut [u8],
    ) -> Result<(), DiskError<Io::Error>> {
        let mbr = MasterBootRecord::protective_mbr(self.io.num_blocks()?);
        self.write_mbr(&mbr, block_buf)
    }

    /// Write an MBR to the first block. If the block size is bigger
    /// than the MBR, the rest of the block will be filled with zeroes.
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn write_mbr(
        &mut self,
        mbr: &MasterBootRecord,
        mut block_buf: &mut [u8],
    ) -> Result<(), DiskError<Io::Error>> {
        block_buf = self.clip_block_buf_size(block_buf)?;

        let mbr_bytes = bytes_of(mbr);

        // This should always be true because the block_buf size is
        // already known to match the block size, and the block size is
        // enforced to be at least 512 bytes which is the size of the
        // MBR struct.
        assert!(block_buf.len() >= mbr_bytes.len());

        {
            let (left, right) = block_buf.split_at_mut(mbr_bytes.len());
            left.copy_from_slice(mbr_bytes);
            right.fill(0);
        }

        self.io.write_blocks(Lba(0), block_buf)?;
        Ok(())
    }

    /// Write the primary GPT header to the second block.
    ///
    /// The header is written to the beginning of the block, and all
    /// remaining bytes in the block are set to zero (see Table 5-5 "GPT
    /// Header" in the UEFI Specification: "The rest of the block is
    /// reserved by UEFI and must be zero").
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn write_primary_gpt_header(
        &mut self,
        header: &GptHeader,
        block_buf: &mut [u8],
    ) -> Result<(), DiskError<Io::Error>> {
        self.write_gpt_header(Lba(1), header, block_buf)
    }

    /// Write the secondary GPT header to the last block.
    ///
    /// The header is written to the beginning of the block, and all
    /// remaining bytes in the block are set to zero (see Table 5-5 "GPT
    /// Header" in the UEFI Specification: "The rest of the block is
    /// reserved by UEFI and must be zero").
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn write_secondary_gpt_header(
        &mut self,
        header: &GptHeader,
        block_buf: &mut [u8],
    ) -> Result<(), DiskError<Io::Error>> {
        let num_blocks = self.io.num_blocks()?;
        let last_block =
            Lba(num_blocks.checked_sub(1).ok_or(DiskError::Overflow)?);
        self.write_gpt_header(last_block, header, block_buf)
    }

    /// Write a [`GptHeader`] to the specified [`Lba`].
    ///
    /// The header is written to the beginning of the block, and all
    /// remaining bytes in the block are set to zero (see Table 5-5 "GPT
    /// Header" in the UEFI Specification: "The rest of the block is
    /// reserved by UEFI and must be zero").
    ///
    /// `block_buf` is a mutable byte buffer with a length of at least one block.
    pub fn write_gpt_header(
        &mut self,
        lba: Lba,
        header: &GptHeader,
        mut block_buf: &mut [u8],
    ) -> Result<(), DiskError<Io::Error>> {
        block_buf = self.clip_block_buf_size(block_buf)?;

        let header_bytes = bytes_of(header);

        // This should always be true because the block_buf size is
        // already known to match the block size, and the block size is
        // enforced to be at least 512 bytes which is much larger than
        // the size of the GptHeader struct.
        assert!(block_buf.len() >= header_bytes.len());

        {
            let (left, right) = block_buf.split_at_mut(header_bytes.len());
            left.copy_from_slice(header_bytes);
            right.fill(0);
        }

        self.io.write_blocks(lba, block_buf)?;
        Ok(())
    }
}

impl<Io: BlockIo> BlockIo for Disk<Io> {
    type Error = Io::Error;

    fn block_size(&self) -> BlockSize {
        self.io.block_size()
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        self.io.num_blocks()
    }

    fn read_blocks(
        &mut self,
        start_lba: Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.io.read_blocks(start_lba, dst)
    }

    fn write_blocks(
        &mut self,
        start_lba: Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        self.io.write_blocks(start_lba, src)
    }

    /// Flush any pending writes to the disk.
    ///
    /// This is called automatically when the disk is dropped, but if an
    /// error occurs at that point it will be silently ignored. It is
    /// recommended to call this method directly before dropping the disk.
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.io.flush()
    }
}

impl<Io: BlockIo> Drop for Disk<Io> {
    fn drop(&mut self) {
        // Throw away any errors.
        let _r = self.flush();
    }
}
