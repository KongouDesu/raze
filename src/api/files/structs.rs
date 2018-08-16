use sha1;
use std;
use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;
use std::cmp::min;
use std::fmt::Formatter;
use std::fmt::Error;

#[derive(Deserialize, Serialize, Debug, Eq, Clone)]
#[serde(rename_all = "camelCase")]
/// Contains information about a file stored on the BackBlaze server
///
/// This is returned when a file is uploaded or the information about a buckets files are requested
///
/// Note: the custom info from the 'file_info' field is not yet implemented and will be ignored
pub struct StoredFile {
    pub file_id: String,
    pub file_name: String,
    pub account_id: Option<String>, // Only present when StoredFile is returned from upload_file
    pub bucket_id: Option<String>,  // Only present when StoredFile is returned from upload_file
    pub content_length: u64,
    pub content_sha1: String,
    pub content_type: String,
    //pub file_info: String,
    pub action: String,
    pub upload_timestamp: u64
}

/// Compares by the file_name value
impl Ord for StoredFile {
    fn cmp(&self, other: &StoredFile) -> Ordering {
        self.file_name.cmp(&other.file_name)
    }
}

/// Compares by the file_name value
impl PartialOrd for StoredFile {
    fn partial_cmp(&self, other: &StoredFile) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

/// Compares by the file_name value
impl PartialEq for StoredFile {
    fn eq(&self, other: &StoredFile) -> bool {
        self.file_name == other.file_name
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Authorization for uploading files
///
/// Authorization is on a per-bucket basis.
/// This stores the associated bucket id and the retrieved upload url as well as it's token
///
/// See also [get_upload_url](../misc/fn.get_upload_url.html)
pub struct UploadAuth {
    pub bucket_id: String,
    pub upload_url: String,
    pub authorization_token: String
}

/// Used to send the hash at the end of the body
///
/// When the hash is provided at the end of the request body, this struct is used \
/// The request will read via. the [Read](https://doc.rust-lang.org/std/io/trait.Read.html) trait. \
/// While uploading the file, the bytes read will also be loaded into the hasher. \
/// When the file upload is done, the hash will be read instead.
/// When both file and hash has been read, this reader is empty
pub(super) struct HashAtEndOfBody {
    file: File,
    hash: sha1::Sha1,
    hash_read: usize,
}

impl std::fmt::Debug for HashAtEndOfBody {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "File: {:?}, hash_read: {}", self.file, self.hash_read)
    }
}

impl HashAtEndOfBody {
    /// Creates the reader used to provide the hash at the end of the body using the given file
    pub fn new(file: File) -> HashAtEndOfBody {
        HashAtEndOfBody {
            file,
            hash: sha1::Sha1::new(),
            hash_read: 0,
        }
    }
}

impl Read for HashAtEndOfBody {
    // This will first read from the file, passing it to the hasher as well as returning it
    // When the entire file is read, this read will instead return the hash
    // When both file and hash has been read, this will return Ok(0) indicating it is finished
    fn read(&mut self, buf: &mut [u8]) -> Result<usize,std::io::Error> {
        // Try to read into the buffer
        let len = self.file.read(buf)?;
        // There are now 3 possible cases

        // We read nothing, but haven't addded hash yet
        if len == 0 && self.hash_read == 0 || self.hash_read > 0 && self.hash_read < 40{
            let digest = self.hash.digest();
            let buf_size = buf.len();
            // If the buffer isn't large enough for our hash, we need to split it up
            if buf_size < 40 || self.hash_read > 0 {
                let end_index = min(40, self.hash_read+buf_size);
                let read_amount = end_index-self.hash_read;
                buf[0..read_amount].clone_from_slice(&format!("{}", digest).as_bytes()[self.hash_read..end_index]);
                self.hash_read += read_amount;
                Ok(read_amount)
            }else{
                // If there's enough room, just send the 40-byte hexdigest as bytes
                buf[0..40].clone_from_slice(&format!("{}", digest).as_bytes());
                self.hash_read = 40;
                Ok(40)
            }
            // We read nothing and we have added the hash, meaning we're done
        }else if len == 0 || self.hash_read == 40{
            Ok(0)
            // There's yet more to read from the file
        }else{
            self.hash.update(&buf[0..len]);
            Ok(len)
        }
    }
}

/// Like [HashAtEndOfBody](struct.HashAtEndOfBody.html), but allows for the upload to be throttled
///
/// bandwidth is the maximum amount of bytes per second sent
///
/// Note: this sleeps the thread uploading this body
pub(super) struct ThrottledHashAtEndOfBody {
    file: File,
    hash: sha1::Sha1,
    hash_read: usize,
    bandwidth: usize,
}

impl std::fmt::Debug for ThrottledHashAtEndOfBody {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "File: {:?}, hash_read: {} - bandwidth: {}", self.file, self.hash_read, self.bandwidth)
    }
}

impl ThrottledHashAtEndOfBody {
    /// Create a new throttled, streaming body. Bandwidth is bytes/sec
    pub fn new(file: File, bandwidth: usize) -> ThrottledHashAtEndOfBody {
        ThrottledHashAtEndOfBody {
            file,
            hash: sha1::Sha1::new(),
            hash_read: 0,
            bandwidth,
        }
    }
}

impl Read for ThrottledHashAtEndOfBody {
    // Much like HashAtEndOfBody, but will sleep in-between reads to limit bandwidth usage
    fn read(&mut self, buf: &mut [u8]) -> Result<usize,std::io::Error> {
        // Try to read into the buffer
        let len = if buf.len() >= 2048 {
            self.file.read(&mut buf[0..2048])?
        } else {
            self.file.read(buf)?
        };
        // There are now 3 possible cases

        // We read nothing, but haven't addded hash yet
        if len == 0 && self.hash_read == 0 || self.hash_read > 0 && self.hash_read < 40{
            let digest = self.hash.digest();
            let buf_size = buf.len();
            // If the buffer isn't large enough for our hash, we need to split it up
            if buf_size < 40 || self.hash_read > 0 {
                let end_index = min(40, self.hash_read+buf_size);
                let read_amount = end_index-self.hash_read;
                buf[0..read_amount].clone_from_slice(&format!("{}", digest).as_bytes()[self.hash_read..end_index]);
                self.hash_read += read_amount;
                Ok(read_amount)
            }else{
                // If there's enough room, just send the 40-byte hexdigest as bytes
                buf[0..40].clone_from_slice(&format!("{}", digest).as_bytes());
                self.hash_read = 40;
                Ok(40)
            }
            // We read nothing and we have added the hash, meaning we're done
        }else if len == 0 || self.hash_read == 40{
            Ok(0)
            // There's yet more to read from the file
        }else{
            self.hash.update(&buf[0..len]);
            // Calculates how much bandwidth we just used as a float from 0 to 1
            let usage = (len as f64)/(self.bandwidth as f64);
            // Sleeps for an appropriate amount
            std::thread::sleep(std::time::Duration::from_millis((usage*1000.) as u64));
            Ok(len)
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Request body used for the [b2_list_file_names](https://www.backblaze.com/b2/docs/b2_list_file_names.html) call
pub(crate) struct ListFileBody {
    pub bucket_id: String,
    pub start_file_name: String,
    pub max_file_count: usize,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Request body for the [b2_update_bucket](https://www.backblaze.com/b2/docs/b2_update_bucket.html) call
pub(crate) struct UpdateBucket<'a> {
    pub account_id: &'a str,
    pub bucket_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket_type: Option<&'a str>,
    pub lifecycle_rules: Vec<LifecycleRules<'a>>
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// sub-struct for UpdateBucket, contains lifecycle rules
pub struct LifecycleRules<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_from_uploading_to_hiding: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_from_hiding_to_deleting: Option<u32>,
    pub file_name_prefix: &'a str,
}

#[derive(Debug, Clone)]
/// Information on a downloaded file
///
/// This comes from headers in a non-json format, so we manually construct this
pub struct FileInfo {
    pub x_bz_file_name: Option<String>,
    pub x_bz_file_id: Option<String>,
    pub x_bz_content_sha1: Option<String>,
    pub x_bz_upload_timestamp: Option<u64>,
    pub x_bz_info_src_last_modified_millis: Option<u64>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
}