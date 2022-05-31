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

//! Library for reading and writing GPT disk data structures through a
//! block IO interface.
//!
//! This crate adds a convenient interface for reading and writing the
//! GPT types defined in the [`gpt_disk_types`] crate to a [`Disk`]. The
//! [`Disk`] is represented by the [`BlockIo`] trait, which allows this
//! library to be `no_std`. The disk can be backed by:
//! * [`SliceBlockIo`]: a read-only byte slice
//! * [`MutSliceBlockIo`]: a mutable byte slice
//! * [`StdBlockIo`] (only available if the `std` feature is enabled):
//!   wraps any type that implements [`Read`] + [`Write`] + [`Seek`],
//!   such as a [`File`].
//! * A custom implementation of the [`BlockIo`] trait.
//!
//! # Features
//!
//! * `std`: Enables the [`StdBlockIo`] type, as well as
//!   `std::error::Error` implementations for all of the error
//!   types. Off by default.
//!
//! [`File`]: std::fs::File
//! [`Read`]: std::io::Read
//! [`Seek`]: std::io::Seek
//! [`Write`]: std::io::Write

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::as_conversions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod block_io;
mod disk;
mod slice_block_io;
#[cfg(feature = "std")]
mod std_support;

// Re-export dependencies.
pub use gpt_disk_types;

pub use block_io::BlockIo;
pub use disk::{Disk, DiskError};
pub use slice_block_io::{MutSliceBlockIo, SliceBlockIo, SliceBlockIoError};

#[cfg(feature = "std")]
pub use std_support::StdBlockIo;
