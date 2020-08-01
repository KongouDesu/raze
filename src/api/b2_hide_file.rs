use reqwest::blocking::Client;
use ::Error;
use api::{B2Auth, B2FileInfo};
use handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HideFileBody<'a> {
    bucket_id: &'a str,
    file_name: &'a str,
}

/// <https://www.backblaze.com/b2/docs/b2_delete_file_version.html>
pub fn b2_hide_file<T: AsRef<str>, Q: AsRef<str>>(client: &Client, auth: &B2Auth, bucket_id: T, file_name: Q) -> Result<B2FileInfo, Error> {
    let req_body = serde_json::to_string(&HideFileBody {
        bucket_id: bucket_id.as_ref(),
        file_name: file_name.as_ref(),
    }).unwrap();

    let resp = match client.post(&auth.api_url_for("b2_hide_file"))
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
    let deserialized: B2FileInfo = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}