use reqwest::blocking::Client;
use ::Error;
use api::{B2Auth, BucketResult};
use handle_b2error_kinds;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ListBucketsBody<'a> {
    account_id: &'a str,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ListBucketsResult {
    pub buckets: Vec<BucketResult>,
}

/// <https://www.backblaze.com/b2/docs/b2_list_buckets.html>
pub fn b2_list_buckets(client: &Client, auth: &B2Auth) -> Result<Vec<BucketResult>, Error> {
    let req_body = serde_json::to_string(&ListBucketsBody {
        account_id: &auth.account_id,
    }).unwrap();

    let resp = match client.post(&auth.api_url_for("b2_list_buckets"))
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
    let deserialized: ListBucketsResult = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized.buckets)
}