// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::util::parse_byte_from_ascii_str_at;
use crate::{Guid, GuidFromStrError};

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

pub(crate) const fn try_parse_guid(s: &str) -> Result<Guid, GuidFromStrError> {
    // Treat input as ASCII.
    let s = s.as_bytes();

    if s.len() != 36 {
        return Err(GuidFromStrError::Length);
    }

    let sep = b'-';
    if s[8] != sep {
        return Err(GuidFromStrError::Separator(8));
    }
    if s[13] != sep {
        return Err(GuidFromStrError::Separator(13));
    }
    if s[18] != sep {
        return Err(GuidFromStrError::Separator(18));
    }
    if s[23] != sep {
        return Err(GuidFromStrError::Separator(23));
    }

    Ok(Guid {
        time_low: [
            mtry!(parse_byte_from_ascii_str_at(s, 6)),
            mtry!(parse_byte_from_ascii_str_at(s, 4)),
            mtry!(parse_byte_from_ascii_str_at(s, 2)),
            mtry!(parse_byte_from_ascii_str_at(s, 0)),
        ],
        time_mid: [
            mtry!(parse_byte_from_ascii_str_at(s, 11)),
            mtry!(parse_byte_from_ascii_str_at(s, 9)),
        ],
        time_high_and_version: [
            mtry!(parse_byte_from_ascii_str_at(s, 16)),
            mtry!(parse_byte_from_ascii_str_at(s, 14)),
        ],
        clock_seq_high_and_reserved: mtry!(parse_byte_from_ascii_str_at(s, 19)),
        clock_seq_low: mtry!(parse_byte_from_ascii_str_at(s, 21)),
        node: [
            mtry!(parse_byte_from_ascii_str_at(s, 24)),
            mtry!(parse_byte_from_ascii_str_at(s, 26)),
            mtry!(parse_byte_from_ascii_str_at(s, 28)),
            mtry!(parse_byte_from_ascii_str_at(s, 30)),
            mtry!(parse_byte_from_ascii_str_at(s, 32)),
            mtry!(parse_byte_from_ascii_str_at(s, 34)),
        ],
    })
}
