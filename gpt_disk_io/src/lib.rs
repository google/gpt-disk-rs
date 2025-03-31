// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Library for reading and writing GPT disk data structures through a
//! block IO interface.
//!
//! This crate adds a convenient interface for reading and writing the
//! GPT types defined in the [`gpt_disk_types`] crate to a [`Disk`]. The
//! [`Disk`] is represented by the [`BlockIo`] trait, which allows this
//! library to be `no_std`. The [`BlockIo`] trait is designed to be
//! compatible with very simple block APIs, such as [`EFI_BLOCK_IO_PROTOCOL`].
//!
//! The [`BlockIoAdapter`] type allows the disk to be backed by simple
//! byte-oriented storage backends, such as `&mut [u8]` and `File` (the
//! latter requires the `std` feature).
//!
//! # Features
//!
//! * `alloc`: Enables [`Vec`] implementation of [`BlockIoAdapter`].
//! * `std`: Enables [`std::io`] implementations of [`BlockIoAdapter`].
//!
//! # Examples
//!
//! Construct a GPT disk in-memory backed by a `Vec`:
//!
//! ```
//! use gpt_disk_io::{BlockIoAdapter, BlockIo, Disk, DiskError};
//! use gpt_disk_types::{
//!     guid, BlockSize, Crc32, GptHeader, GptPartitionEntry,
//!     GptPartitionEntryArray, GptPartitionType, LbaLe, U32Le,
//! };
//!
//! // Space for a 4MiB disk.
//! let mut disk_storage = vec![0; 4 * 1024 * 1024];
//!
//! // Standard 512-byte block size.
//! let bs = BlockSize::BS_512;
//!
//! // `BlockIoAdapter` implements the `BlockIo` trait which is used by
//! // the `Disk` type for reading and writing.
//! let block_io = BlockIoAdapter::new(disk_storage.as_mut_slice(), bs);
//!
//! let mut disk = Disk::new(block_io)?;
//!
//! // Manually construct the header and partition entries.
//! let primary_header = GptHeader {
//!     header_crc32: Crc32(U32Le::from_u32(0xa4877843)),
//!     my_lba: LbaLe::from_u64(1),
//!     alternate_lba: LbaLe::from_u64(8191),
//!     first_usable_lba: LbaLe::from_u64(34),
//!     last_usable_lba: LbaLe::from_u64(8158),
//!     disk_guid: guid!("57a7feb6-8cd5-4922-b7bd-c78b0914e870"),
//!     partition_entry_lba: LbaLe::from_u64(2),
//!     number_of_partition_entries: U32Le::from_u32(128),
//!     partition_entry_array_crc32: Crc32(U32Le::from_u32(0x9206adff)),
//!     ..Default::default()
//! };
//! let secondary_header = GptHeader {
//!     header_crc32: Crc32(U32Le::from_u32(0xdbeb4c13)),
//!     my_lba: LbaLe::from_u64(8191),
//!     alternate_lba: LbaLe::from_u64(1),
//!     partition_entry_lba: LbaLe::from_u64(8159),
//!     ..primary_header
//! };
//! let partition_entry = GptPartitionEntry {
//!     partition_type_guid: GptPartitionType(guid!(
//!         "ccf0994f-f7e0-4e26-a011-843e38aa2eac"
//!     )),
//!     unique_partition_guid: guid!("37c75ffd-8932-467a-9c56-8cf1f0456b12"),
//!     starting_lba: LbaLe::from_u64(2048),
//!     ending_lba: LbaLe::from_u64(4096),
//!     attributes: Default::default(),
//!     name: "hello world!".parse().unwrap(),
//! };
//!
//! // Create a buffer the length of one block. A `Vec` is used here,
//! // but any mutable byte slice with the right length will do.
//! let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
//!
//! // Write out the protective MBR and GPT headers. Note that without
//! // the protective MBR, some tools won't recognize the disk as GPT.
//! disk.write_protective_mbr(&mut block_buf)?;
//! disk.write_primary_gpt_header(&primary_header, &mut block_buf)?;
//! disk.write_secondary_gpt_header(&secondary_header, &mut block_buf)?;
//!
//! // Construct the partition entry array.
//! let layout = primary_header.get_partition_entry_array_layout().unwrap();
//! let mut bytes =
//!     vec![0; layout.num_bytes_rounded_to_block_as_usize(bs).unwrap()];
//! let mut entry_array =
//!     GptPartitionEntryArray::new(layout, bs, &mut bytes).unwrap();
//! *entry_array.get_partition_entry_mut(0).unwrap() = partition_entry;
//!
//! // Write the primary partition entry array.
//! disk.write_gpt_partition_entry_array(&entry_array)?;
//!
//! // Write the secondary partition entry array.
//! entry_array.set_start_lba(secondary_header.partition_entry_lba.into());
//! disk.write_gpt_partition_entry_array(&entry_array)?;
//!
//! // Ensure all writes are flushed. This is not needed with the slice
//! // backend, but is good practice for "real" IO. (The disk will also
//! // flush when dropped, but any errors at that point are ignored.)
//! disk.flush()?;
//!
//! # Ok::<(), gpt_disk_io::DiskError<gpt_disk_io::SliceBlockIoError>>(())
//! ```
//!
//! [`File`]: std::fs::File
//! [`Read`]: std::io::Read
//! [`Seek`]: std::io::Seek
//! [`Write`]: std::io::Write
//! [`EFI_BLOCK_IO_PROTOCOL`]: https://uefi.org/specs/UEFI/2.10/13_Protocols_Media_Access.html#block-i-o-protocol

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::as_conversions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod block_io;
mod disk;

// Re-export dependencies.
pub use gpt_disk_types;

pub use block_io::slice_block_io::SliceBlockIoError;
pub use block_io::{BlockIo, BlockIoAdapter};
pub use disk::{Disk, DiskError};

#[cfg(feature = "std")]
pub use block_io::std_block_io::ReadWriteSeek;
