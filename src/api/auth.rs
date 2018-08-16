use reqwest;
use std;
use std::io::Read;
use base64::{encode};
use B2Error;
use serde_json;
use handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Contains the authorization response
pub struct B2Auth {
    pub account_id: String,
    pub api_url: String,
    pub authorization_token: String,
    pub download_url: String,
    pub absolute_minimum_part_size: usize,
    pub recommended_part_size: usize
}

/// Takes auth as a string in the format "keyId:applicationKey" and attempts to authenticate
pub fn authenticate(client: &reqwest::Client, auth: &str) -> Result<B2Auth, B2Error> {
    let encoded = format!("{}{}", "Basic ", encode(&auth));

    let mut head = reqwest::header::Headers::new();
    head.set(reqwest::header::Authorization(encoded));

    let mut resp = match client.get("https://api.backblazeb2.com/b2api/v1/b2_authorize_account")
        .headers(head)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    if resp.status().is_client_error() {
        return Err(B2Error::from_response(resp))
    }
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: B2Auth = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

/// Like [authenticate](fn.authenticate.html), but takes a path to a file instead of a string
///
/// The file must be a single line containing "keyId:applicationKey" without quotes
pub fn authenticate_from_file(client: &reqwest::Client, file: &std::path::Path) -> Result<B2Auth, B2Error> {
    let mut read =  match std::fs::File::open(file) {
        Ok(v) => v,
        Err(e) => return Err(B2Error::IOError(e))
    };
    let mut contents = String::new();
    if let Err(e) = read.read_to_string(&mut contents) { return Err(B2Error::IOError(e)) }
    let auth_str = contents.trim();

    authenticate(client,&auth_str)
}

#[cfg(test)]
mod tests {
    use api::auth;
    use std;
    use reqwest;
    use ::tests::TEST_CREDENTIALS_FILE as TEST_CREDENTIALS_FILE;

    // This also tests 'authenticate', as 'authenticate_from_file' calls it
    #[test]
    fn test_auth() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE));
        match autho {
            Ok(_b2_auth) => (),
            Err(b2_error) => {
                println!("Help: does a valid credentials file exist in project root?");
                println!("Help: see project-level docs for credentials file help");
                println!("This test must pass for almost every other test to pass");
                println!("Error: {:?}",b2_error);
                assert!(false);
            }
        }
    }
}
