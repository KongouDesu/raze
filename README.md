# raze
Rust API bindings for the BackBlaze B2 API.
Provides raw API calls or an easy-to-use struct via the engine module.

The calls are documented and should be pretty obvious if you've read the [official docs][1]

There is an example project using this library to make a simple CLI backup tool: [raze-cli][2]

This library is a proof of concept to show simple API calls. Refer to https://crates.io/crates/backblaze-b2 for a complete implementation.

   [1]: https://www.backblaze.com/b2/docs/
   [2]: https://github.com/KongouDesu/raze-cli/tree/master

# API implementation status
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
b2_delete_file_version          | 🚧
b2_delete_key                   | ❌
b2_download_file_by_id          | ?
b2_download_file_by_name        | ?
b2_finish_large_file            | ❌
b2_get_download_authorization   | ?
b2_get_file_info                | ?
b2_get_upload_part_url          | ❌
b2_get_upload_url               | 🚧
b2_hide_file                    | 🚧
b2_list_buckets                 | ✔
b2_list_file_names              | 🚧
b2_list_file_versions           | ?
b2_list_keys                    | ❌
b2_list_parts                   | ❌
b2_list_unfinished_large_files  | ❌
b2_start_large_file             | ❌
b2_update_bucket                | ✔
b2_upload_file                  | 🚧
b2_upload_part                  | ❌