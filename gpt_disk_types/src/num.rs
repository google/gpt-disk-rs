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

use bytemuck::{Pod, Zeroable};
use core::fmt::{self, Debug, Display, Formatter, LowerHex};

/// 16-bit unsigned integer stored as a little-endian.
#[derive(
    Clone, Copy, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable,
)]
#[repr(transparent)]
pub struct U16Le(pub [u8; 2]);

impl U16Le {
    /// Convert to [`u16`] with the host's endianness.
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        u16::from_le_bytes(self.0)
    }

    /// Create a `U16Le` from a [`u16`] with the host's endianness.
    #[must_use]
    pub const fn from_u16(v: u16) -> Self {
        Self(v.to_le_bytes())
    }

    /// Update the value to a [`u16`] with the host's endianness.
    pub fn set(&mut self, v: u16) {
        *self = Self::from_u16(v);
    }
}

impl Debug for U16Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_u16(), f)
    }
}

impl Display for U16Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_u16(), f)
    }
}

impl LowerHex for U16Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format_u8_slice_lower_hex_le(f, &self.0)
    }
}

/// 32-bit unsigned integer stored as a little-endian.
#[derive(
    Clone, Copy, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable,
)]
#[repr(transparent)]
pub struct U32Le(pub [u8; 4]);

impl U32Le {
    /// Convert to [`u32`] with the host's endianness.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    /// Create a `U32Le` from a [`u32`] with the host's endianness.
    #[must_use]
    pub const fn from_u32(v: u32) -> Self {
        Self(v.to_le_bytes())
    }

    /// Update the value to a [`u32`] with the host's endianness.
    pub fn set(&mut self, v: u32) {
        *self = Self::from_u32(v);
    }
}

impl Debug for U32Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_u32(), f)
    }
}

impl Display for U32Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_u32(), f)
    }
}

impl LowerHex for U32Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format_u8_slice_lower_hex_le(f, &self.0)
    }
}

/// 64-bit unsigned integer stored as a little-endian.
#[derive(
    Clone, Copy, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable,
)]
#[repr(transparent)]
pub struct U64Le(pub [u8; 8]);

impl U64Le {
    /// Convert to [`u64`] with the host's endianness.
    #[must_use]
    pub const fn to_u64(self) -> u64 {
        u64::from_le_bytes(self.0)
    }

    /// Create a `U64Le` from a [`u64`] with the host's endianness.
    #[must_use]
    pub const fn from_u64(v: u64) -> Self {
        Self(v.to_le_bytes())
    }

    /// Update the value to a [`u64`] with the host's endianness.
    pub fn set(&mut self, v: u64) {
        *self = Self::from_u64(v);
    }
}

impl Debug for U64Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.to_u64(), f)
    }
}

impl Display for U64Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.to_u64(), f)
    }
}

impl LowerHex for U64Le {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format_u8_slice_lower_hex_le(f, &self.0)
    }
}

pub(crate) fn format_u8_slice_lower_hex_le(
    f: &mut Formatter<'_>,
    s: &[u8],
) -> fmt::Result {
    if f.alternate() {
        f.write_str("0x")?;
    }
    for byte in s.iter().rev() {
        write!(f, "{:02x}", byte)?;
    }
    Ok(())
}
