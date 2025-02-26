# 0.16.1

* Derive `PartialEq` for `DiskError`.

# 0.16.0

* Bump MSRV to 1.68.
* Relax the requirements of `BlockIo::read_blocks` and
  `BlockIo::write_blocks` such that reading/writing zero blocks is
  allowed.
* Add `BlockIoAdapter<T>`. This struct impls `BlockIo` for types that
  don't have a block size. For example, `&mut [u8]`, `Vec<u8>`, and
  `File`.
* Add `alloc` feature. This enables the `BlockIoAdapter<Vec<u8>>` impl.
* Remove `SliceBlockIo`, `MutSliceBlockIo`, and `StdBlockIo`. Use
  `BlockIoAdapter` instead.
* Remove `BlockIo::assert_valid_buffer`. Use
  `BlockSize::assert_valid_block_buffer` instead.
* Update to latest `gpt_disk_types`.

# 0.15.0

* Update to latest `gpt_disk_types`.
* Copied the license files into each package so that the archives on
  crates.io include them.

# 0.14.0

* Relax version requirement for `bytemuck`.
* When the `std` feature is enabled, also enable it in `gpt_disk_types`.
* Enable `doc_auto_cfg` on docs.rs.
* Updated to latest `gpt_disk_types`.

# 0.13.0

* Allow the MIT license to be used in addition to Apache-2.0.
* Updated to latest `gpt_disk_types`.

# 0.12.0

* Updated to latest `gpt_disk_types`.

# 0.11.0

* Updated to latest `gpt_disk_types`.

# 0.10.0

* Updated to latest `gpt_disk_types`.

# 0.9.0

* Updated to latest `gpt_disk_types`.

# 0.8.0

* Updated to latest `gpt_disk_types`.

# 0.7.0

* Updated to latest `gpt_disk_types`.

# 0.6.0

* Updated documentation.
* Updated to latest `gpt_disk_types`.
