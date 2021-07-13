use reqwest::blocking::Client;

use crate::api::{B2Auth, B2FileInfo};
use crate::handle_b2error_kinds;
use crate::Error;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct GetFileInfoBody<'a> {
    pub file_id: &'a str,
}

/// Gets the info the B2 has about the file with the given ID
///
/// file_id is the ID obtained when uploading or from b2_list_file_names
/// Note that b2_list_file_names already returns the file info, so if you use that, there is no need to call this
///
/// <https://www.backblaze.com/b2/docs/b2_get_file_info.html>
pub fn b2_get_file_info<T: AsRef<str>>(
    client: &Client,
    auth: &B2Auth,
    file_id: T,
) -> Result<B2FileInfo, Error> {
    let req_body = serde_json::to_string(&GetFileInfoBody {
        file_id: file_id.as_ref(),
    })
    .unwrap();

    let resp = match client
        .post(&auth.api_url_for("b2_get_file_info"))
        .header(reqwest::header::AUTHORIZATION, &auth.authorization_token)
        .body(req_body)
        .send()
    {
        Ok(v) => v,
        Err(e) => return Err(Error::ReqwestError(e)),
    };
    if !resp.status().is_success() {
        return Err(Error::from_response(resp));
    }

    let response_string = resp.text().unwrap();
    let deserialized: B2FileInfo = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string));
        }
    };
    Ok(deserialized)
}
