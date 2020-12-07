# raze
[![Crates.io](https://img.shields.io/crates/v/raze)](https://crates.io/crates/raze)
[![Documentation](https://docs.rs/raze/badge.svg)](https://docs.rs/raze/)

Rust API bindings for the BackBlaze B2 API.
Provides raw API calls along with some helpful utilities.

The raw API calls are more or less 1:1 with the [official B2 docs][1]

   [1]: https://www.backblaze.com/b2/docs/

Disclaimer: This library is not associated with Backblaze - Be aware of the [B2 pricing](https://www.backblaze.com/b2/cloud-storage-pricing.html) - Refer to License.md for conditions

## API implementation status
 * ✔️ - Implemented
 * 🚧 - Planned
 * ❌ - Not planned

If you need something that isn't implemented, open an issue or submit a pull request  
Note that many features marked 'optional' by Backblaze are of much lower priority than implementing the rest of the API  

Name | Status
---- | ------
b2_authorize_account            | ✔
b2_cancel_large_file            | ❌
b2_copy_file                    | ❌
b2_copy_part                    | ❌
b2_create_bucket                | ✔
b2_create_key                   | ❌
b2_delete_bucket                | ✔
b2_delete_file_version          | ✔
b2_delete_key                   | ❌
b2_download_file_by_id          | 🚧
b2_download_file_by_name        | 🚧
b2_finish_large_file            | ❌
b2_get_download_authorization   | 🚧
b2_get_file_info                | 🚧
b2_get_upload_part_url          | ❌
b2_get_upload_url               | ✔
b2_hide_file                    | ✔
b2_list_buckets                 | ✔
b2_list_file_names              | ✔
b2_list_file_versions           | 🚧
b2_list_keys                    | ❌
b2_list_parts                   | ❌
b2_list_unfinished_large_files  | ❌
b2_start_large_file             | ❌
b2_update_bucket                | ✔
b2_upload_file                  | ✔
b2_upload_part                  | ❌
