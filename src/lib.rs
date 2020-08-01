//! Raze is a library for interfacing the [BackBlaze B2 API](https://www.backblaze.com/b2/cloud-storage.html)
//!
//! Raze provides raw API bindings via the [API](api/index.html) along with some useful functions via [utils](util/index.html) \
//! It is highly recommended to familiarize yourself with the [official B2 documentation](https://www.backblaze.com/b2/docs/) before using this crate \
//!
//! This crate exposes a **blocking** API by the use of [reqwest](https://crates.io/crates/reqwest/0.10.7) \
//! Note that despite being blocking, the same `Client` can be shared by multiple threads for concurrent usage, and it is recommended to do so for uploading multiple files at once
//!
//! Disclaimer: This library is not associated with Backblaze - Be aware of the [B2 pricing](https://www.backblaze.com/b2/cloud-storage-pricing.html) - Refer to License.md for conditions
//!
//! ## Example:
//! ```rust
//! # use raze::api::*;
//! # use raze::util::*;
//! // Authenticate, upload and delete a file
//! let client = reqwest::blocking::ClientBuilder::new().timeout(None).build().unwrap();
//! let auth = authenticate_from_file(&client, "credentials").unwrap();
//! let upauth = b2_get_upload_url(&client, &auth, "bucket_id").unwrap();
//! let file = std::fs::File::open("document.txt").unwrap();
//! let metadata = file.metadata().unwrap();
//! let size = metadata.len();
//! let modf = metadata.modified().unwrap()
//!                 .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()*1000;
//!
//! let param = FileParameters {
//!     file_path: "document.txt",
//!     file_size: size,
//!     content_type: None,
//!     content_sha1: Sha1Variant::HexAtEnd,
//!     last_modified_millis: modf,
//! };
//!
//! let reader = file;
//! let reader = ReadHashAtEnd::wrap(reader);
//! let reader = ReadThrottled::wrap(reader, 5000);
//!
//! let resp1 = b2_upload_file(&client, &upauth, reader, param).unwrap();
//!
//! let resp2 = b2_delete_file_version(&client, &auth, &resp1.file_name, &resp1.file_id.unwrap());
//! ```



extern crate reqwest;
extern crate base64;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate sha1;
extern crate url;

/// Raw API bindings, mostly 1:1 with official API
pub mod api;
/// Various helper functions to assist with common tasks
pub mod util;

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
            Err(e) => return Error::SerdeError(e)
        };
        Error::B2Error(deserialized)
    }

    /// Same as from_string but works directly on a reqwest::Response
    fn from_response(resp: reqwest::blocking::Response) -> Error {
        match resp.text() {
            Ok(s) =>  Error::from_json(&s),
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
    pub message: String
}

impl fmt::Debug for B2ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "B2ApiError: error code {} - {}. Message: {}",self.status, self.code, self.message)
    }
}

impl fmt::Display for B2ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "A B2 API Error occurred. Error code {} - {}. Error message: {}",self.status, self.code, self.message)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::*;
    use crate::util::*;

    #[test]
    fn test_dev() {
        let client = reqwest::blocking::ClientBuilder::new().timeout(None).build().unwrap();
        let auth = authenticate_from_file(&client, "credentials").unwrap();
        println!("{:?}",auth);
        let upauth = b2_get_upload_url(&client, &auth, "bucket_id").unwrap();
        println!("{:?}", upauth);
        let file = std::fs::File::open("Cargo.lock").unwrap();
        let size = file.metadata().unwrap().len();
        let modf = file.metadata().unwrap().modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()*1000;

        let param = FileParameters {
            file_path: "Cargo.lock",
            file_size: size,
            content_type: None,
            content_sha1: Sha1Variant::HexAtEnd,
            last_modified_millis: modf,
        };

        let reader= file;
        let reader = ReadHashAtEnd::wrap(reader);
        let reader = ReadThrottled::wrap(reader, 10000);

        let t = std::time::Instant::now();
        let resp1 = b2_upload_file(&client, &upauth, reader, param);
        println!("{:?}", resp1);
        println!("Upload took {}", t.elapsed().as_secs_f32());
        let resp1 = resp1.unwrap();

        let resp2 = b2_delete_file_version(&client, &auth, &resp1.file_name, &resp1.file_id.unwrap());
        println!("{:?}", resp2);

    }
}