/// Methods for uploading files. Large file API is currently not supported
pub mod upload;
/// Various API calls not yet grouped into categories
pub mod misc;
/// Contains various structs used by the file API
///
/// Most of these structs are internal-only, used to serialize requests
pub mod structs;
/// Methods for downloading files. Large file API is currently not supported
pub mod download;