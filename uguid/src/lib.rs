// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Library providing a GUID (Globally Unique Identifier) type. The
//! format is defined in [RFC 4122]. However, unlike "normal" UUIDs
//! (such as those provided by the [`uuid`] crate), the first three
//! fields are little-endian. See [Appendix A] of the UEFI
//! Specification. This format of GUID is also used in Microsoft
//! Windows.
//!
//! [Appendix A]: https://uefi.org/specs/UEFI/2.10/Apx_A_GUID_and_Time_Formats.html
//! [RFC 4122]: https://datatracker.ietf.org/doc/html/rfc4122
//! [`uuid`]: https://docs.rs/uuid/latest/uuid
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
//! ##[rustfmt::skip]
//! let guid1 = Guid::from_bytes([
//!     0x01, 0x02, 0x03, 0x04,
//!     0x05, 0x06, 0x07, 0x08,
//!     0x09, 0x10, 0x11, 0x12,
//!     0x13, 0x14, 0x15, 0x16,
//! ]);
//! let guid2 = Guid::new(
//!     [0x01, 0x02, 0x03, 0x04],
//!     [0x05, 0x06],
//!     [0x07, 0x08],
//!     0x09,
//!     0x10,
//!     [0x11, 0x12, 0x13, 0x14, 0x15, 0x16],
//! );
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

/// Macro replacement for the `?` operator, which cannot be used in
/// const functions.
macro_rules! mtry {
    ($expr:expr $(,)?) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return Err(err);
            }
        }
    };
}

mod error;
mod guid;
mod util;

pub use error::GuidFromStrError;
pub use guid::{Guid, Variant};

#[cfg(feature = "std")]
impl std::error::Error for GuidFromStrError {}

/// Create a [`Guid`] from a string at compile time.
///
/// # Examples
///
/// ```
/// use uguid::{guid, Guid};
/// assert_eq!(
///     guid!("01234567-89ab-cdef-0123-456789abcdef"),
///     Guid::new(
///         [0x67, 0x45, 0x23, 0x01],
///         [0xab, 0x89],
///         [0xef, 0xcd],
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
        const G: $crate::Guid = $crate::Guid::parse_or_panic($s);
        G
    }};
}
