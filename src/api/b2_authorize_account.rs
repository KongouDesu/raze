use reqwest::blocking::Client;
use base64::{encode};
use Error;
use serde_json;
use handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// An authorization from [b2_authorize_account](fn.b2_authorize_account.html) - Required for most other calls
///
/// Note: 'allowed' object is currently *unsupported*
pub struct B2Auth {
    pub account_id: String,
    pub authorization_token: String,
    pub api_url: String,
    pub download_url: String,
    pub absolute_minimum_part_size: usize,
    pub recommended_part_size: usize
}

impl B2Auth {
    // Given the name of an api call, return the full url for it
    // See https://www.backblaze.com/b2/docs/calling.html
    pub fn api_url_for(&self, call_name: &str) -> String {
        format!("{}/b2api/v2/{}",self.api_url,call_name)
    }
}

/// Authenticate with the API - B2Auth is required by other commands
///
/// 'keystring' is a string with the format "applicationKeyId:applicationKey" (Remember the colon)
///
/// <https://www.backblaze.com/b2/docs/b2_authorize_account.html>
pub fn b2_authorize_account<T: AsRef<str>>(client: &Client, keystring: T) -> Result<B2Auth, Error> {
    // Encode the key
    let encoded = format!("{}{}", "Basic ", encode(keystring.as_ref()));

    // Submit the request
    let resp = match client.get("https://api.backblazeb2.com/b2api/v2/b2_authorize_account")
        .header(reqwest::header::AUTHORIZATION, encoded)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(Error::ReqwestError(e))
    };
    // If it didn't succeed, return ReqwestError
    if !resp.status().is_success() {
        return Err(Error::from_response(resp))
    }

    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Attempt to deserialize the JSON
    // There are 3 cases here
    // 1. API call succeeded and it deserializes to a B2Auth struct
    // 2. API call succeeded but response is an API Error - returns B2Error
    // 3. API call went through, but response matches neither B2Auth nor B2Error - returns SerdeError
    let deserialized: B2Auth = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}