# 0.16.0

* Bump MSRV to 1.68.
* Add `BlockSize::is_multiple_of_block_size`.
* Add `BlockSize::assert_valid_block_buffer`.
* Add `LbaRangeInclusive::num_blocks`.
* Documentation improvements.

# 0.15.0

* Updated to latest `uguid`.
* The `Guid` type's alignment has changed from 1 to 4.
* The `GptHeader` and `GptPartitionEntry` structs are now `repr(C, packed)`
  instead of just `repr(C)`. This is due to the alignment of `Guid` changing
  from 1 to 4.
* Copied the license files into each package so that the archives on
  crates.io include them.

# 0.14.0

* Relax version requirement for `bytemuck`.
* When the `std` feature is enabled, also enable it in `uguid`.
* Added a `bytemuck` feature (disabled by default). All of the code that
  depends on the `bytemuck` crate is now gated behind this feature.
* Enable `doc_auto_cfg` on docs.rs.

# 0.13.0

* Allow the MIT license to be used in addition to Apache-2.0.

# 0.12.0

* Updated to latest `uguid`.

# 0.11.0

* Updated to latest `uguid`.
* Remove re-export of `bytemuck` dependency.

# 0.10.0

* Updated to latest `uguid`.

# 0.9.0

* Add dependency on `uguid`. The `Guid` and `GuidFromStrError` types, as
  well as the `guid!` macro, are now provided by `uguid` and re-exported
  by `gpt_disk_types`.

# 0.8.0

* Added `Guid::to_bytes`
* Added `LbaRangeInclusive::num_bytes`.

# 0.7.0

* Added `Guid::try_parse`. This is a `const` method that is functionally
  equivalent to `Guid::from_str`.
* Added `guid!` macro for creating a `Guid` from a string at compile time.
* Added several `GptPartitionType` constants: `EFI_SYSTEM`,
  `LEGACY_MBR`, `BASIC_DATA`, `CHROME_OS_KERNEL`, and
  `CHROME_OS_ROOT_FS`.

# 0.6.0

* Renamed the `BlockSize` constants: `B512`→`BS_512` and
  `B4096`→`BS_4096`. The previous names were a little hard to read.
* Updated documentation.
