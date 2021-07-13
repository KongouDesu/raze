//! Raze is a library for interfacing the [BackBlaze B2 API](https://www.backblaze.com/b2/cloud-storage.html)
//!
//! Raze provides raw API bindings via the [API][api] along with some useful functions via [util]. \
//! It is highly recommended to familiarize yourself with the [official B2 documentation](https://www.backblaze.com/b2/docs/) before using this crate. \
//!
//! This crate exposes an **async** API by the use of [tokio] and [reqwest].
//!
//! Disclaimer: This library is not associated with Backblaze - Be aware of the [B2 pricing](https://www.backblaze.com/b2/cloud-storage-pricing.html) - Refer to License.md for conditions
//!
//! ## Example:
//! ```rust
//! # use raze::api::*;
//! # use raze::util::*;
//!
//! // Authenticate, upload and delete a file
//! #[tokio::main]
//! async fn main() {
//!     let client = reqwest::ClientBuilder::new().build().unwrap();
//!     let auth = b2_authorize_account(&client, std::env::var("B2_TEST_KEY_STRING").unwrap()).await.unwrap();
//!     let upauth = b2_get_upload_url(&client, &auth, std::env::var("B2_TEST_BUCKET_ID").unwrap()).await.unwrap();
//!     let file = tokio::fs::File::open("tests/resources/simple_text_file.txt").await.unwrap();
//!     let metadata = file.metadata().await.unwrap();
//!     let size = metadata.len();
//!     let modf = metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()*1000;
//!
//!     let param = FileParameters {
//!         file_path: "simple_text_file.txt",
//!         file_size: size,
//!         content_type: None,
//!         content_sha1: Sha1Variant::HexAtEnd,
//!         last_modified_millis: modf,
//!     };
//!
//!     let reader = file;
//!     let reader = AsyncReadHashAtEnd::wrap(reader);
//!     let reader = AsyncReadThrottled::wrap(reader, 5000);
//!     let body = body_from_reader(reader);
//!
//!     let resp1 = b2_upload_file(&client, &upauth, body, param).await.unwrap();
//!
//!     let resp2 = b2_delete_file_version(&client, &auth, &resp1.file_name, &resp1.file_id.unwrap()).await.unwrap();
//! }
//! ```

/// Raw API bindings, mostly 1:1 with official API
pub mod api;
/// Various helper functions to assist with common tasks
pub mod util;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
/// The various kinds of errors this crate may return
pub enum Error {
    /// HTTP related errors
    ReqwestError(reqwest::Error),
    /// IO related errors
    IOError(std::io::Error),
    /// (De)Serialization related errors
    SerdeError(serde_json::Error),
    /// API related errors, returned by the B2 backend
    B2Error(B2ApiError),
}

impl Error {
    /// Constructs a B2Error from a json string
    ///
    /// When we get an API error, we get an error message as a string \
    /// This will create a B2Error containing that string
    ///
    /// In case the error message is invalid/unexpected JSON, this returns a SerdeError instead
    fn from_json(error: &str) -> Error {
        let deserialized: B2ApiError = match serde_json::from_str(error) {
            Ok(v) => v,
            Err(e) => return Error::SerdeError(e),
        };
        Error::B2Error(deserialized)
    }

    /// Same as from_string but works directly on a reqwest::Response
    async fn from_response(resp: reqwest::Response) -> Error {
        match resp.text().await {
            Ok(s) => Error::from_json(&s),
            Err(e) => Error::ReqwestError(e),
        }
    }
}

// Helper method for figuring out if an error was a Serde or API error
// Takes the json-str, return either a B2 API error or a Serde error
fn handle_b2error_kinds(n: &str) -> Error {
    let _b2err: B2ApiError = match serde_json::from_str(&n) {
        Ok(v) => return Error::B2Error(v),
        Err(e) => return Error::SerdeError(e),
    };
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// An API error, returned by the B2 backend
///
/// You typically run into this in 2 cases:
/// 1. The backend ran into some issue, e.g. timed out, your authorization expired etc. \
/// 2. Wrong use of the API, e.g. malformed args, insufficient permissions
///
/// Official documentation: [Error Handling](https://www.backblaze.com/b2/docs/calling.html#error_handling)
pub struct B2ApiError {
    /// Contains the HTTP response code
    pub status: u16,
    /// A short string name for the error, eg. "invalid_bucket_name"
    pub code: String,
    /// A human-readable error message describing what went wrong
    pub message: String,
}

impl fmt::Debug for B2ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "B2ApiError: error code {} - {}. Message: {}",
            self.status, self.code, self.message
        )
    }
}

impl fmt::Display for B2ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "A B2 API Error occurred. Error code {} - {}. Error message: {}",
            self.status, self.code, self.message
        )
    }
}
