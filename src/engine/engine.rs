//! The engine itself, providing simplified API access

use reqwest;
use api::*;
use api::files::*;
use B2Error;
use std;
use std::path::Path;
use api::files::structs::*;
use api::buckets::{Bucket,BucketType};

#[derive(Debug, Clone)]
/// An object-like struct.
/// This is the engine itself and all calls are made through this
pub struct Raze {
    b2auth: Option<auth::B2Auth>,
    active_bucket: Option<String>,
    upload_auth: Option<UploadAuth>,
    client: reqwest::Client,
}

impl Raze {
    /// Create a new, not yet authenticated, engine
    pub fn new() -> Raze {
        let c = reqwest::Client::builder()
            // The timeout is set arbitrarily high to prevent slow uploads from timing out
            // The BackBlaze backend can still timeout requests for other reasons
            .timeout(std::time::Duration::new(100000000,0))
            .build()
            .unwrap();

        Raze {
            b2auth: None,
            active_bucket: None,
            upload_auth: None,
            client: c
        }
    }

    /// Create an engine from an existing [B2Auth](../../api/auth/struct.B2Auth.html), obtained from raw API calls
    pub fn from_auth(auth: auth::B2Auth) -> Raze {
        let c = reqwest::Client::new();
        Raze {
            b2auth: Some(auth),
            active_bucket: None,
            upload_auth: None,
            client: c,
        }
    }

    /// Authenticate this instance with an auth string like "keyId:applicationKey"
    ///
    /// See also the [raw authenticate](../../api/auth/fn.authenticate.html)
    pub fn authenticate(&mut self, auth: &str) -> Option<B2Error> {
        let auth = match auth::authenticate(&self.client, &auth) {
            Ok(b2_auth) => b2_auth,
            Err(e) => return Some(e),
        };
        self.b2auth = Some(auth);
        None
    }

    /// Same as [authenticate](fn.authenticate.html), but takes a path to a file containing only the auth string
    ///
    /// See also the [raw authenticate from file](../../api/auth/fn.authenticate_from_file.html)
    pub fn authenticate_from_file(&mut self, file: &std::path::Path) -> Option<B2Error> {
        let auth = match auth::authenticate_from_file(&self.client, &file) {
            Ok(b2_auth) => b2_auth,
            Err(e) => return Some(e),
        };
        self.b2auth = Some(auth);
        None
    }

    /// Attempts to get an upload auth
    ///
    /// Internally uses [get_upload_url](../../api/misc/fn.get_upload_url.html)
    ///
    /// This function is called by the engine itself when it needs to
    fn authenticate_uploading(&mut self) -> Result<UploadAuth,B2Error> {
        let auth = match &self.b2auth {
            Some(ref a) => a,
            None => return Err(B2Error::B2EngineError),
        };
        let bucket = match &self.active_bucket {
            Some(ref a) => a,
            None => return Err(B2Error::B2EngineError),
        };
        let up_auth = match misc::get_upload_url(&self.client, &auth, &bucket) {
            Ok(up_auth) => up_auth,
            Err(e) => return Err(e),
        };
        Ok(up_auth)
    }

    /// Sets which bucket we're uploading to
    ///
    /// bucket_id is the internal id that BackBlaze assigns a bucket
    ///
    /// Note that this function doesn't verify that this is a valid bucket!
    pub fn set_active_bucket(&mut self, bucket_id: String) {
        self.active_bucket = Some(bucket_id);
    }

    /// Attempts to upload a given file
    ///
    /// This will get upload authentication if needed
    ///
    /// This will upload to the currently active bucket \
    /// The prefix is added in front of the filename if you want to emulate folders, see the [raw upload_file](../../api/files/upload/fn.upload_file.html)
    pub fn upload_file(&mut self, file_path: &Path, prefix: &str) -> Result<StoredFile,B2Error> {
        let upload_auth = match self.authenticate_uploading() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        upload::upload_file(&self.client, &upload_auth, &file_path, prefix)
    }

    /// Like [upload_file](struct.Raze.html#method.upload_file), but will use a streaming read to save memory
    ///
    /// See also: [raw upload_file_streaming](../../api/files/upload/fn.upload_file_streaming.html)
    pub fn upload_file_streaming(&mut self, file_path: &Path, prefix: &str) -> Result<StoredFile,B2Error> {
        let upload_auth = match self.authenticate_uploading() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        upload::upload_file_streaming(&self.client, &upload_auth, &file_path, prefix)
    }

    /// Like [upload_file_streaming](struct.Raze.html#method.upload_file_streaming), but allows you to throttle the upload
    ///
    /// Throttling limits the upload to at most 'bandwidth' bytes per second
    ///
    /// See also: [raw upload_file_throttled](../../api/files/upload/fn.upload_file_throttled.html)
    pub fn upload_file_throttled(&mut self, file_path: &Path, prefix: &str, bandwidth: usize) -> Result<StoredFile,B2Error> {
        let upload_auth = match self.authenticate_uploading() {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        upload::upload_file_throttled(&self.client, &upload_auth, &file_path, prefix, bandwidth)
    }

    /// Delete the specified version of the given file
    ///
    /// See also: [raw delete_file_version](../../api/files/misc/fn.delete_file_version.html)
    pub fn delete_file_version(&mut self, file_name: String, file_id: String) -> Option<B2Error> {
        let auth = match &self.b2auth {
            Some(ref a) => a,
            None => return Some(B2Error::B2EngineError),
        };
        match misc::delete_file_version(&self.client, &auth, file_name, file_id) {
            Err(e) => Some(e),
            _ => None,
        }
    }

    /// Lists available buckets for the authenticated account
    ///
    /// See also: [raw list_buckets](../../api/buckets/fn.list_buckets.html)
    pub fn list_buckets(&mut self) -> Result<Vec<buckets::Bucket>,B2Error> {
        let auth = match &self.b2auth {
            Some(ref a) => a,
            None => return Err(B2Error::B2EngineError),
        };
        match buckets::list_buckets(&self.client, auth) {
            Ok(v) => Ok(v),
            Err(e) => Err(e)
        }
    }

    /// Gets a list of all files in the currently active bucket
    ///
    /// Will retrieve 'max_file_count' files per call \
    /// Please see [raw list_all_file_names](../../api/files/misc/fn.list_all_file_names.html) for what this implies
    pub fn list_all_file_names(&mut self, bucket_id: &str, max_file_count: usize) -> Result<Vec<StoredFile>, B2Error> {
        let auth = match &self.b2auth {
            Some(ref a) => a,
            None => return Err(B2Error::B2EngineError),
        };
        misc::list_all_file_names(&self.client, auth, bucket_id, max_file_count)
    }

    /// Updates the type and days until hiding or deleting files
    ///
    /// See also: [raw update_bucket](../../api/buckets/fn.update_bucket.html)
    pub fn update_bucket(&mut self, bucket_id: &str, bucket_type: Option<BucketType>, days_from_hide_to_delete: Option<u32>, days_from_upload_to_hide: Option<u32>) -> Result<Bucket, B2Error> {
        let auth = match &self.b2auth {
            Some(ref a) => a,
            None => return Err(B2Error::B2EngineError),
        };
        match buckets::update_bucket(&self.client, auth, bucket_id, bucket_type, days_from_hide_to_delete, days_from_upload_to_hide) {
            Ok(b) => Ok(b),
            Err(e) => Err(e),
        }
    }
}

impl Default for Raze {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use engine::engine;
    use std;
    use std::io::Read;
    use ::tests::TEST_CREDENTIALS_FILE as TEST_CREDENTIALS_FILE;
    use ::tests::TEST_BUCKET_ID as TEST_BUCKET_ID;

    // Tests for the various to get an authenticated engine
    #[test]
    fn test_auth_from_file() {
        let mut r = engine::Raze::new();
        r.authenticate_from_file(std::path::Path::new(TEST_CREDENTIALS_FILE));
    }

    #[test]
    fn test_auth_from_str() {
        let mut r = engine::Raze::new();
        // Read the auth file
        let mut read = std::fs::File::open(TEST_CREDENTIALS_FILE).unwrap();
        let mut contents = String::new();
        read.read_to_string(&mut contents).unwrap();
        r.authenticate(&contents);
    }

    use api::auth;
    use reqwest;

    #[test]
    #[allow(unused_variables)]
    fn test_from_auth() {
        let client = reqwest::Client::new();
        let auth = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let r = engine::Raze::from_auth(auth);
    }


    // Test that we can upload and delete files
    #[test]
    #[allow(unused_variables)]
    fn test_upload_and_delete() {
        let mut r = engine::Raze::new();
        r.authenticate_from_file(std::path::Path::new(TEST_CREDENTIALS_FILE));
        r.set_active_bucket(TEST_BUCKET_ID.to_string());
        let n = r.upload_file(&std::path::Path::new("./testfile.txt"), "").unwrap();
        r.delete_file_version(n.file_name, n.file_id);
    }

    // Test that we can upload streaming
    #[test]
    #[allow(unused_variables)]
    fn test_upload_streaming() {
        let mut r = engine::Raze::new();
        r.authenticate_from_file(std::path::Path::new(TEST_CREDENTIALS_FILE));
        r.set_active_bucket(TEST_BUCKET_ID.to_string());
        let n = r.upload_file_streaming(&std::path::Path::new("./testfile.txt"), "").unwrap();
        r.delete_file_version(n.file_name, n.file_id);
    }

    // Test that we can get a list of buckets
    // Soft test, only verifies it doesn't crash
    #[test]
    #[allow(unused_variables)]
    fn test_list_buckets() {
        let mut r = engine::Raze::new();
        r.authenticate_from_file(std::path::Path::new(TEST_CREDENTIALS_FILE));
        let list = r.list_buckets().unwrap();
    }

    #[test]
    #[allow(unused_variables)]
    // This will test than update_bucket call can succeed
    // By default this will call on the first bucket in your account, but the settings are all configured to not changed anything
    // Still, don't blame me if it changes something you didn't expect :)
    fn test_update_bucket_engine() {
        let mut r = engine::Raze::new();
        r.authenticate_from_file(std::path::Path::new(TEST_CREDENTIALS_FILE));
        let bucko = &r.list_buckets().unwrap()[0].bucket_id;
        use api::buckets::BucketType;
        let n = r.update_bucket(&bucko, None, None, None);
        match n {
            Ok(n) => (),
            Err(e) => assert!(false),
        }
    }
}