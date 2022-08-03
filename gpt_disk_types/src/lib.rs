// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Library of GPT disk data types.
//!
//! # GPT disk components
//!
//! ```text
//! ┌───┬───────┬─────────────────┬─────────┬───────────────────┬─────────┐
//! │MBR│Primary│Primary partition│Partition│Secondary partition│Secondary│
//! │   │header │entry array      │data     │entry array        │header   │
//! └───┴───────┴─────────────────┴─────────┴───────────────────┴─────────┘
//! ```
//!
//! 1. The first block of the disk contains a protective MBR. See
//! [`MasterBootRecord::protective_mbr`].
//! 2. The second block of the disk contains the primary GPT header. See
//! [`GptHeader`].
//! 3. Additional blocks after the header contain the partition entry
//! array. See [`GptPartitionEntry`] and [`GptPartitionEntryArray`].
//! 4. At the end of the disk is a secondary GPT header and partition
//! entry array.
//!
//! # Endianness
//!
//! The UEFI Specification specifies that data structures are little
//! endian (section 1.8.1 "Data Structure Descriptions"). Unless
//! otherwise noted, all fields in this library are little endian. This
//! is true even when running the code on a big-endian architecture; the
//! [`U16Le`], [`U32Le`], [`U64Le`], and [`LbaLe`] types help enforce
//! this. The little-endian convention is also used for [`Display`]
//! implementations. This means bytes within each field will appear
//! reversed when compared with a flat hex dump of GPT data.
//!
//! [`Display`]: core::fmt::Display
//!
//! # Features
//!
//! * `std`: Provides `std::error::Error` implementations for all of the
//!   error types. Off by default.
//!
//! # Examples
//!
//! Construct a GPT header:
//!
//! ```
//! use gpt_disk_types::{guid, Crc32, GptHeader, LbaLe, U32Le};
//!
//! let header = GptHeader {
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
//! ```
//!
//! Construct a GPT partition entry:
//!
//! ```
//! use gpt_disk_types::{guid, GptPartitionEntry, GptPartitionType, LbaLe};
//!
//! let entry = GptPartitionEntry {
//!     partition_type_guid: GptPartitionType(guid!(
//!         "ccf0994f-f7e0-4e26-a011-843e38aa2eac"
//!     )),
//!     unique_partition_guid: guid!("37c75ffd-8932-467a-9c56-8cf1f0456b12"),
//!     starting_lba: LbaLe::from_u64(2048),
//!     ending_lba: LbaLe::from_u64(4096),
//!     attributes: Default::default(),
//!     name: "hello world!".parse().unwrap(),
//! };
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::as_conversions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

mod block;
mod crc32;
mod header;
mod mbr;
mod num;
mod partition_array;
mod partition_entry;
#[cfg(feature = "std")]
mod std_support;

// Re-export dependencies.
pub use crc;
pub use ucs2;
pub use uguid::{guid, Guid, GuidFromStrError};

pub use block::{BlockSize, Lba, LbaLe, LbaRangeInclusive};
pub use crc32::Crc32;
pub use header::{GptHeader, GptHeaderRevision, GptHeaderSignature};
pub use mbr::{Chs, DiskGeometry, MasterBootRecord, MbrPartitionRecord};
pub use num::{U16Le, U32Le, U64Le};
pub use partition_array::{
    GptPartitionEntryArray, GptPartitionEntryArrayError,
    GptPartitionEntryArrayLayout,
};
pub use partition_entry::{
    GptPartitionAttributes, GptPartitionEntry, GptPartitionEntrySize,
    GptPartitionEntrySizeError, GptPartitionName, GptPartitionNameFromStrError,
    GptPartitionNameSetCharError, GptPartitionType,
};
