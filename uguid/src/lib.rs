// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Library providing a GUID (Globally Unique Identifier) type. The
//! format is described in Appendix A of the UEFI
//! Specification. This format of GUID is also used in Microsoft
//! Windows.
//!
//! Two versions of the GUID struct are provided that are identical
//! except for the struct alignment. [`Guid`] is 1-byte aligned, and
//! [`AlignedGuid`] is 8-byte aligned. These types can be conveniently
//! constructed with the [`guid!`] and [`aligned_guid!`] macros.
//!
//! # Features
//!
//! No features are enabled by default.
//!
//! * `bytemuck`: Implements bytemuck's `Pod` and `Zeroable` traits for `Guid`.
//! * `serde`: Implements serde's `Serialize` and `Deserialize` traits for `Guid`.
//! * `std`: Provides `std::error::Error` implementation for the error type.
//!
//! # Examples
//!
//! Construct a GUID at compile time with the `guid!` macro:
//!
//! ```
//! use uguid::guid;
//!
//! let guid = guid!("01234567-89ab-cdef-0123-456789abcdef");
//! ```
//!
//! Parse a GUID at runtime from a string:
//!
//! ```
//! use uguid::Guid;
//!
//! let guid: Guid = "01234567-89ab-cdef-0123-456789abcdef".parse().unwrap();
//! ```
//!
//! Construct a GUID from its components or a byte array:
//!
//! ```
//! use uguid::Guid;
//!
//! let guid1 = Guid::new(
//!     0x01234567_u32.to_le_bytes(),
//!     0x89ab_u16.to_le_bytes(),
//!     0xcdef_u16.to_le_bytes(),
//!     0x01,
//!     0x23,
//!     [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
//! );
//! let guid2 = Guid::from_bytes([
//!     0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67,
//!     0x89, 0xab, 0xcd, 0xef,
//! ]);
//! assert_eq!(guid1, guid2);
//! ```
//!
//! Convert to a string or a byte array:
//!
//! ```
//! use uguid::guid;
//!
//! let guid = guid!("01234567-89ab-cdef-0123-456789abcdef");
//! assert_eq!(guid.to_string(), "01234567-89ab-cdef-0123-456789abcdef");
//! assert_eq!(
//!     guid.to_bytes(),
//!     [
//!         0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45,
//!         0x67, 0x89, 0xab, 0xcd, 0xef
//!     ]
//! );
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::as_conversions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod guid_impl;
mod util;

pub use error::GuidFromStrError;
pub use guid_impl::{AlignedGuid, Guid};

#[cfg(feature = "std")]
impl std::error::Error for GuidFromStrError {}

/// Create an unaligned [`Guid`] from a string at compile time.
///
/// # Examples
///
/// ```
/// use uguid::{guid, Guid};
/// assert_eq!(
///     guid!("01234567-89ab-cdef-0123-456789abcdef"),
///     Guid::new(
///         0x01234567_u32.to_le_bytes(),
///         0x89ab_u16.to_le_bytes(),
///         0xcdef_u16.to_le_bytes(),
///         0x01,
///         0x23,
///         [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
///     )
/// );
/// ```
#[macro_export]
macro_rules! guid {
    ($s:literal) => {{
        // Create a temporary const value to force an error in the input
        // to fail at compile time.
        const g: $crate::Guid = $crate::Guid::parse_or_panic($s);
        g
    }};
}

/// Create an [`AlignedGuid`] from a string at compile time.
///
/// # Examples
///
/// ```
/// use uguid::{aligned_guid, AlignedGuid};
/// assert_eq!(
///     aligned_guid!("01234567-89ab-cdef-0123-456789abcdef"),
///     AlignedGuid::new(
///         0x01234567_u32.to_le_bytes(),
///         0x89ab_u16.to_le_bytes(),
///         0xcdef_u16.to_le_bytes(),
///         0x01,
///         0x23,
///         [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
///     )
/// );
/// ```
#[macro_export]
macro_rules! aligned_guid {
    ($s:literal) => {{
        // Create a temporary const value to force an error in the input
        // to fail at compile time.
        const g: $crate::AlignedGuid = $crate::AlignedGuid::parse_or_panic($s);
        g
    }};
}
