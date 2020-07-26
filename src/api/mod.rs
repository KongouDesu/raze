///! Raw API calls

// Auth is used elsewhere, export it
pub use self::b2_authorize_account::B2Auth;

/// The types a bucket can have
/// Note that 'Snapshot' cannot be created via b2_create_bucket or b2_update_bucket
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum B2BucketType {
    AllPublic,
    AllPrivate,
    Snapshot,
}

/// API response from 'b2_create_bucket', 'b2_update_bucket', and 'b2_delete_bucket'
/// Also returned as part of an array w/ 'b2_list_buckets'
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BucketResult {
    pub account_id: String,
    pub bucket_id: String,
    pub bucket_name: String,
    pub bucket_type: B2BucketType,
}

/// API response from 'b2_upload_file' and 'b2_hide_file'
/// 'b2_list_file_names' and 'b2_list_file_versions' returns a list of them
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct B2FileInfo {
    pub account_id: String,
    pub action: String,
    pub bucket_id: String,
    pub content_length: u64,
    pub content_sha1: Option<String>,
    pub content_type: Option<String>,
    pub file_id: Option<String>,
    pub file_name: String,
    pub upload_timestamp: u64,
}

// Export API calls
mod b2_authorize_account;
pub use self::b2_authorize_account::*;

mod b2_create_bucket;
pub use self::b2_create_bucket::*;
mod b2_update_bucket;
pub use self::b2_update_bucket::*;
mod b2_delete_bucket;
pub use self::b2_delete_bucket::*;
mod b2_list_buckets;
pub use self::b2_list_buckets::*;

mod b2_list_file_names;
pub use self::b2_list_file_names::*;

mod b2_get_upload_url;
pub use self::b2_get_upload_url::*;
mod b2_upload_file;
pub use self::b2_upload_file::*;
mod b2_delete_file_version;
pub use self::b2_delete_file_version::*;
mod b2_hide_file;
pub use self::b2_hide_file::*;