/// The types a bucket can have
///
/// Note that 'Snapshot' cannot be created via b2_create_bucket or b2_update_bucket
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum B2BucketType {
    AllPublic,
    AllPrivate,
    Snapshot,
}

/// Represents a 'Bucket' on B2
///
/// API response from 'b2_create_bucket', 'b2_update_bucket', 'b2_delete_bucket' and 'b2_list_buckets'
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct BucketResult {
    pub account_id: String,
    pub bucket_id: String,
    pub bucket_name: String,
    pub bucket_type: B2BucketType,
}

/// Represents a file on B2
///
/// API response from 'b2_upload_file' and 'b2_hide_file', 'b2_list_file_names' and 'b2_list_file_versions'
#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
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

/// Compares by the file_name value
impl Ord for B2FileInfo {
    fn cmp(&self, other: &B2FileInfo) -> Ordering {
        self.file_name.cmp(&other.file_name)
    }
}

impl PartialOrd for B2FileInfo {
    fn partial_cmp(&self, other: &B2FileInfo) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

/// Compares by the file_name value
impl PartialEq for B2FileInfo {
    fn eq(&self, other: &B2FileInfo) -> bool {
        self.file_name == other.file_name
    }
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
use std::cmp::Ordering;