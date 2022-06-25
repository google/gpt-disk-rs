# 1.0.0

* Make `GuidFromStrError` into an enum with three variants to allow for
  better error messages.

# 0.7.0

* Add a const `Guid::from_bytes` constructor.
* Make `Guid::to_bytes` const.
* Remove re-export of `bytemuck` dependency.
* Make the `bytemuck` dependency optional with the new `bytemuck` feature.

# 0.6.0

* Add `Guid::to_ascii_hex_lower` method. This is a const function that
  creates a `[u8; 36]` array containing the GUID in standard
  `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` format.
* Add `serde` feature (disabled by default) that implements serde's
  `Serialize` and `Deserialize` traits for the `Guid` type.
* Remove unused `From<ParseIntError>` impl for `GuidFromStrError`.
