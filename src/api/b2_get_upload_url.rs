use reqwest::blocking::Client;
use ::Error;
use api::{B2Auth};
use handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct GetUploadUrlBody<'a> {
    bucket_id: &'a str,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Authorization and upload url
/// Needed for b2_upload_file
pub struct UploadAuth {
    pub bucket_id: String,
    pub upload_url: String,
    pub authorization_token: String,
}

/// https://www.backblaze.com/b2/docs/b2_get_upload_url.html
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