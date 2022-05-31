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
//! One notable exception is the [`Guid`] type, which is partially
//! little endian and partially big endian as described in Appendix A of
//! the UEFI Specification.
//!
//! # Features
//!
//! * `std`: Provides `std::error::Error` implementations for all of the
//!   error types. Off by default.
//!
//! [`Display`]: core::fmt::Display

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
mod guid;
mod header;
mod mbr;
mod num;
mod partition_array;
mod partition_entry;
#[cfg(feature = "std")]
mod std_support;

// Re-export dependencies.
pub use bytemuck;
pub use crc;
pub use ucs2;

pub use block::{BlockSize, Lba, LbaLe, LbaRangeInclusive};
pub use crc32::Crc32;
pub use guid::{Guid, GuidFromStrError};
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
