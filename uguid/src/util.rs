// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub(crate) const fn byte_to_ascii_hex_lower(byte: u8) -> (u8, u8) {
    let mut l = byte & 0xf;
    let mut h = byte >> 4;
    if l <= 9 {
        l += b'0';
    } else {
        l += b'a' - 10;
    }
    if h <= 9 {
        h += b'0';
    } else {
        h += b'a' - 10;
    }
    (h, l)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ascii() {
        assert_eq!(byte_to_ascii_hex_lower(0x1f), (b'1', b'f'));
        assert_eq!(byte_to_ascii_hex_lower(0xf1), (b'f', b'1'));
    }
}
