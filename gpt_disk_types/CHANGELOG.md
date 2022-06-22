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
