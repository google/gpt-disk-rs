# 0.7.0

## `gpt_disk_types`

* Added `Guid::try_parse`. This is a `const` method that is functionally
  equivalent to `Guid::from_str`.
* Added `guid!` macro for creating a `Guid` from a string at compile time.
* Added several `GptPartitionType` constants: `EFI_SYSTEM`,
  `LEGACY_MBR`, `BASIC_DATA`, `CHROME_OS_KERNEL`, and
  `CHROME_OS_ROOT_FS`.

## `gpt_disk_io`

* Updated to latest `gpt_disk_types`.

# 0.6.0

## `gpt_disk_types`

* Renamed the `BlockSize` constants: `B512`→`BS_512` and
  `B4096`→`BS_4096`. The previous names were a little hard to read.
* Updated documentation.

## `gpt_disk_io`

* Updated documentation.
* Updated to latest `gpt_disk_types`.
