use reqwest::blocking::Client;
use crate::{Error, B2ApiError};
use crate::api::{B2Auth, BucketResult};
use crate::handle_b2error_kinds;

/// Authorization used to download files from a bucket
/// Required by b2_download_file_by_name and b2_download_file_by_id
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct B2DownloadAuth {
    pub bucket_id: String,
    pub file_name_prefix: String,
    pub authorization_token: String,
}

/// Parameters for the request
///
/// The B2Auth token must have the 'shareFiles' capability
/// The valid duration must be in the range 1-604800 (1 second to 1 week)
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct B2GetDownloadAuthParams {
    pub bucket_id: String,
    pub file_name_prefix: String,
    pub valid_duration_in_seconds: u32,
}

/// <https://www.backblaze.com/b2/docs/b2_get_download_authorization.html>
pub fn b2_get_download_authorization(client: &Client, auth: &B2Auth, params: B2GetDownloadAuthParams) -> Result<B2DownloadAuth, Error> {
    let req_body = serde_json::to_string(&params).unwrap();

    let resp = match client.post(&auth.api_url_for("b2_get_download_authorization"))
        .header(reqwest::header::AUTHORIZATION, &auth.authorization_token)
        .body(req_body)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(Error::ReqwestError(e))
    };
    if !resp.status().is_success() {
        return Err(Error::from_response(resp))
    }

    let response_string = resp.text().unwrap();
    let deserialized: B2DownloadAuth = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}