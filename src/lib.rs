//! Raze is a library for interfacing the [BackBlaze B2 API](https://www.backblaze.com/b2/docs/)
//!
//! Raze provides raw API bindings via the [API](api/index.html) module or an easy-to-use object-like struct via [engine](engine/index.html) \
//! It is highly recommended to thoroughly read the BackBlaze B2 API documentation before using this crate

extern crate reqwest;
extern crate base64;
#[macro_use] extern crate hyper;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate sha1;
extern crate url;

use std::fmt;
use std::io::Read;

#[derive(Debug)]
/// The various kinds of errors this crate may return
pub enum B2Error {
    /// HTTP related errors
    ReqwestError(reqwest::Error),
    /// IO related errors
    IOError(std::io::Error),
    /// (De)Serialization related errors
    SerdeError(serde_json::Error),
    /// API related errors, returned by the B2 backend
    B2Error(B2ApiError),
}

impl B2Error {
    /// Constructs a B2Error from a json string
    ///
    /// When we get an API error, we get an error message as a string \
    /// This will create a B2Error containing that string
    ///
    /// In case the error message is invalid JSON, this returns a SerdeError instead
    fn from_json(error: &str) -> B2Error {
        let deserialized: B2ApiError = match serde_json::from_str(error) {
            Ok(v) => v,
            Err(e) => return B2Error::SerdeError(e)
        };
        B2Error::B2Error(deserialized)
    }

    /// Same as [from_string](fn.from_json.html) but works directly on a reqwest::Response
    fn from_response(mut resp: reqwest::Response) -> B2Error {
        let mut res = String::new();
        match resp.read_to_string(&mut res) {
            Err(e) => return B2Error::IOError(e),
            _ => ()
        }
        B2Error::from_json(&res)
    }
}

// Helper method for figuring out if an error was a Serde or API error
// Takes the json-str, return either a B2 API error or a Serde error
fn handle_b2error_kinds(n: &str) -> B2Error {
    let _b2err: B2ApiError = match serde_json::from_str(&n) {
        Ok(v) => return B2Error::B2Error(v),
        Err(e) => return B2Error::SerdeError(e),
    };
}

#[derive(Deserialize, Serialize)]
/// An API error, returned by the B2 backend
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