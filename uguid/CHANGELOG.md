# 0.6.0

* Add `Guid::to_ascii_hex_lower` method. This is a const function that
  creates a `[u8; 36]` array containing the GUID in standard
  `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` format.
* Add `serde` feature (disabled by default) that implements serde's
  `Serialize` and `Deserialize` traits for the `Guid` type.
* Remove unused `From<ParseIntError>` impl for `GuidFromStrError`.
