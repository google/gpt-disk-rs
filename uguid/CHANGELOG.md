# 1.2.0 (unreleased)

* Add `Guid::parse_or_panic` and `AlignedGuid::parse_or_panic`. These
  have the same functionality as the corresponding `try_parse` methods,
  except they will panic on failure. This is useful in `const` contexts
  where the panic is used as a compilation error.
* The `guid!` and `aligned_guid!` macros now force const evaluation of
  the input. This was the intended behavior before, but it was not
  implemented correctly. Any new compilation failures caused by this
  change indicate a bug in the calling code.

# 1.1.1

* Change `Guid` back to `repr(C)` instead of `repr(C, align(1))`. Even
  though the alignment of the struct is 1-byte either way, structs with
  any alignment set are not allowed in packed structures so this was a
  breaking change.

# 1.1.0 (yanked)

* Add `AlignedGuid`, which is identical to `Guid` except the struct is
  8-byte aligned instead of 1-byte aligned.
* The `Guid` and `AlignedGuid` types implement `From` for each other to
  convert between them.
* Add `aligned_guid!` macro, which is identical to the `guid!` macro
  except it creates an `AlignedGuid` instead of a `Guid`.
  
This release was yanked due to accidentally changing the repr of `Guid`.

# 1.0.4

* Relax version requirements for `bytemuck` and `serde`.
* Enable `doc_auto_cfg` on docs.rs.

# 1.0.3

* Fix license links in README, take two.

# 1.0.2

* Fix license links in README.

# 1.0.1

* Allow the MIT license to be used in addition to Apache-2.0.

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
