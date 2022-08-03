// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::Guid;
use core::fmt;
use core::str;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ascii = self.to_ascii_hex_lower();
        // OK to unwrap since the ascii output is valid utf-8.
        let s = str::from_utf8(&ascii).unwrap();
        serializer.serialize_str(s)
    }
}

struct GuidDeserializeVisitor;

impl<'de> Visitor<'de> for GuidDeserializeVisitor {
    type Value = Guid;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a string in the format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Guid::try_parse(value).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Guid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(GuidDeserializeVisitor)
    }
}
