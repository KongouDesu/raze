use reqwest::blocking::Client;
use std::path::Path;
use api::B2Auth;
use Error;
use std::io::Read;
use api;

/// Authenticate directly from a file containing credentials
/// The key-file must have the same format as b2_authorize_account expects: "applicationKeyId:applicationKey"
pub fn authenticate_from_file<T: AsRef<Path>>(client: &Client, file: T) -> Result<B2Auth, Error> {
    let mut auth_file = match std::fs::File::open(file) {
        Ok(file) => file,
        Err(e) => return Err(Error::IOError(e)),
    };
    let mut auth = String::with_capacity(60);
    match auth_file.read_to_string(&mut auth) {
        Ok(_) => (),
        Err(e) => return Err(Error::IOError(e)),
    }
    let auth = auth.trim();

    api::b2_authorize_account(client, auth)
}