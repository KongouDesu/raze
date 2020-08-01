use reqwest::blocking::Client;
use ::Error;
use api::{B2Auth, B2FileInfo};
use handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ListFileNamesBody<'a> {
    bucket_id: &'a str,
    start_file_name: &'a str,
    max_file_count: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Contains up to `max_file_count` files and potentially where to continue from with [b2_list_file_names](fn.b2_list_file_names.html)
pub struct ListFilesResult {
    pub files: Vec<B2FileInfo>,
    pub next_file_name: Option<String>,
}

/// <https://www.backblaze.com/b2/docs/b2_list_file_names.html>
///
/// Note billing behavior regarding 'max_file_count' \
/// Leaving 'start_file_name' empty will go from the first file \
/// May return a 'next_file_name' which can be used to continue from where the previous call ended
pub fn b2_list_file_names<T: AsRef<str>, Q: AsRef<str>>(client: &Client, auth: &B2Auth, bucket_id: T, start_file_name: Q, max_file_count: u32) -> Result<ListFilesResult, Error> {
    let req_body = serde_json::to_string(&ListFileNamesBody {
        bucket_id: bucket_id.as_ref(),
        start_file_name: start_file_name.as_ref(),
        max_file_count,
    }).unwrap();

    let resp = match client.post(&auth.api_url_for("b2_list_file_names"))
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
    let deserialized: ListFilesResult = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}