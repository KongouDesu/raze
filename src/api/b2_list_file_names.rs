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
pub struct ListFilesResult {
    pub files: Vec<B2FileInfo>,
    pub next_file_name: Option<String>,
}

/// https://www.backblaze.com/b2/docs/b2_create_bucket.html
/// Note billing behavior regarding 'max_file_count'
pub fn b2_list_file_names<T: AsRef<str>>(client: &Client, auth: &B2Auth, bucket_id: T, start_file_name: T, max_file_count: u32) -> Result<ListFilesResult, Error> {
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