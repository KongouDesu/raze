//! Raze is a library for interfacing the [BackBlaze B2 API](https://www.backblaze.com/b2/docs/)
//!
//! Raze provides raw API bindings via the [API](api/index.html) module or an easy-to-use object-like struct via [engine](engine/index.html) \
//! It is highly recommended to thoroughly read the BackBlaze B2 API documentation before using this crate
//!
//! An example project using this library is available at https://github.com/KongouDesu/raze-cli/tree/master


// Running tests: tests require an auth file, by default 'credentials' located in the root of this project \
// The file contains a single line, formatted as "keyId:applicationKey" without quotes \
// The name and location of the file can be changed by editing it in the 'tests' section of this file
// Additionally, the tests work on a bucket. This must be configured in the tests of lib.rs (this file)
// To configure it, set the constants appropriately

extern crate reqwest;
extern crate base64;
#[macro_use] extern crate hyper;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate sha1;
extern crate url;

pub mod api;
pub mod engine;

use std::fmt;
use std::io::Read;

#[derive(Debug)]
/// The various kinds of errors this crate may return
pub enum B2Error {
    /// HTTP related errors
    ReqwestError(reqwest::Error),
    /// IO related errors
    IOError(std::io::Error),
    /// Serialization related errors
    SerdeError(serde_json::Error),
    /// API related errors, eg. invalid requests
    B2Error(B2ApiError),
    /// An error caused by passing invalid input to one of this library's functions
    InputError(String),
    /// Errors returned from the engine module
    B2EngineError,
}

impl B2Error {
    /// Constructs a B2Error from a json string
    ///
    /// When we get an API error, we get an error message as a string \
    /// This will create a B2Error containing that string
    ///
    /// In case the error message is invalid JSON, this returns a SerdeError instead
    fn from_string(error: &str) -> B2Error {
        let deserialized: B2ApiError = match serde_json::from_str(error) {
            Ok(v) => v,
            Err(e) => return B2Error::SerdeError(e)
        };
        B2Error::B2Error(deserialized)
    }

    /// Same as [from_string](fn.from_string.html) but works directly on a reqwest::Response
    fn from_response(mut resp: reqwest::Response) -> B2Error {
        let mut res = String::new();
        if let Err(e) = resp.read_to_string(&mut res) { return B2Error::IOError(e) }
        B2Error::from_string(&res)
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

#[derive(Deserialize, Serialize, Clone)]
/// An API error, most likely from a bad API call
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

impl std::error::Error for B2ApiError {
    fn description(&self) -> &str {
        "An error caused by some B2 API issue"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    pub const TEST_CREDENTIALS_FILE: &str = "credentials";
    pub const TEST_BUCKET_ID: &str = "4663eedc6289033066000e13";
    pub const TEST_BUCKET_NAME: &str = "uploaded-files";
    use B2Error;

    // Test that a B2ApiError (B2Error::B2Error) is properly deserialized from response json
    #[test]
    fn test_json_to_api_error() {
        let json = r#"{
            "code": "bad_auth_token",
            "message": "Invalid authorization token",
            "status": 401
        }"#;
        // Fails if it isn't a B2Error::B2Error (A B2ApiError)
        let err = match B2Error::from_string(&json) {
            B2Error::B2Error(b2_api_error) => Ok(b2_api_error),
            _ => Err("B2Error is not a B2Error::B2Error")
        }.unwrap();
        // Verify struct contains the same as the input json
        assert_eq!("bad_auth_token", err.code);
        assert_eq!("Invalid authorization token", err.message);
        assert_eq!(401, err.status);
    }

    use api::auth;
    use reqwest;

    // This tests the B2Error::from_response method
    // Implicitly tests from_string, since from_response calls from_string
    // Tries to auth with an invalid auth, then checks if we got a proper auth error
    #[test]
    fn test_invalid_auth_returns_api_error(){
        let client = reqwest::Client::new();
        let response = auth::authenticate(&client, "");
        match response {
            Ok(_b2_auth) => (),
            Err(b2_error) => {
                match b2_error {
                    B2Error::B2Error(err) => {
                        // err.code and err.message has been known to be inconsistent
                        // As thus, we don't test them here
                        assert_eq!(401, err.status);
                    }
                    _ => assert!(false)
                }
            }
        }
    }
}