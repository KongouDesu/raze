use reqwest::blocking::Client;
use std::path::Path;
use api::B2Auth;
use Error;
use std::io::Read;
use api;

/// Authenticate directly from a file containing credentials
///
/// The key-file must have the same format as [b2_authorize_account](../api/fn.b2_authorize_account.html) expects: "applicationKeyId:applicationKey"
pub fn authenticate_from_file<T: AsRef<Path>>(client: &Client, file: T) -> Result<B2Auth, Error> {
    let auth  = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => return Err(Error::IOError(e)),
    };
    let auth = auth.trim();

    api::b2_authorize_account(client, auth)
}