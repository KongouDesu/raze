/// Methods for uploading files. Large file API is currently not supported
pub mod upload;
/// Various API calls not yet grouped into categories
pub mod misc;
/// Contains various structs used by the file API
///
/// Most of these structs are internal-only, used to serialize requests
pub mod structs;
/// Methods for downloading files. Large file API is currently not supported
///
/// Download methods return a ([FileInfo][2], [reqwest::Response][1]) tuple \
/// The [FileInfo][2] struct contains the decoded headers
///
/// The [reqwest::Response][1] is returned immediately upon the request completing and will not have downloaded the whole file \
/// The Response provides a method [copy_to](https://docs.rs/reqwest/0.8.6/reqwest/struct.Response.html#method.copy_to) taking a writer \
/// Calling this method with a writer will start writing to it until the Response reaches EOF \
/// This can be used on, most likely, a file handle, which will then stream write the body to the file
///
/// ```rust,ignore
/// let mut by_id = download_file_by_id(&client, &autho, "file_id")?;
///
/// let mut file = File::create("test_file_name")?;
/// by_id.1.copy_to(&mut file);
/// ```
///
/// [1]: https://docs.rs/reqwest/0.8.6/reqwest/struct.Response.html
/// [2]: ../structs/struct.FileInfo.html
pub mod download;