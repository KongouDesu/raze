use reqwest::blocking::Client;
use crate::Error;
use crate::api::{B2Auth};
use crate::handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct GetUploadUrlBody<'a> {
    bucket_id: &'a str,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
/// Authorization and URL for uploading with [b2_upload_file](fn.b2_upload_file.html) - Distinct from B2Auth
///
/// Note that this should **NOT** be shared - each concurrent upload needs its own UploadAuth
/// Needed for [b2_upload_file](fn.b2_upload_file.html)
pub struct UploadAuth {
    pub bucket_id: String,
    pub upload_url: String,
    pub authorization_token: String,
}

/// <https://www.backblaze.com/b2/docs/b2_get_upload_url.html>
pub fn b2_get_upload_url<T: AsRef<str>>(client: &Client, auth: &B2Auth, bucket_id: T) -> Result<UploadAuth, Error> {
    let req_body = serde_json::to_string(&GetUploadUrlBody {
        bucket_id: bucket_id.as_ref(),
    }).unwrap();

    let resp = match client.post(&auth.api_url_for("b2_get_upload_url"))
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
    let deserialized: UploadAuth = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}