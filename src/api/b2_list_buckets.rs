use crate::api::{B2Auth, BucketResult};
use crate::handle_b2error_kinds;
use crate::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ListBucketsBody<'a> {
    account_id: &'a str,
    bucket_id: Option<String>,
    bucket_name: Option<String>,
    bucket_types: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ListBucketsResult {
    pub buckets: Vec<BucketResult>,
}

/// Represents the optional parameters
pub struct ListBucketParams {
    pub bucket_id: Option<String>,
    pub bucket_name: Option<String>,
    pub bucket_types: Option<String>,
}

/// <https://www.backblaze.com/b2/docs/b2_list_buckets.html>
pub async fn b2_list_buckets(
    client: &Client,
    auth: &B2Auth,
    params: ListBucketParams,
) -> Result<Vec<BucketResult>, Error> {
    let req_body = serde_json::to_string(&ListBucketsBody {
        account_id: &auth.account_id,
        bucket_id: params.bucket_id,
        bucket_name: params.bucket_name,
        bucket_types: params.bucket_types,
    })
    .unwrap();

    let resp = match client
        .post(&auth.api_url_for("b2_list_buckets"))
        .header(reqwest::header::AUTHORIZATION, &auth.authorization_token)
        .body(req_body)
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(Error::ReqwestError(e)),
    };
    if !resp.status().is_success() {
        return Err(Error::from_response(resp).await);
    }

    let response_string = resp.text().await.unwrap();
    let deserialized: ListBucketsResult = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string));
        }
    };
    Ok(deserialized.buckets)
}
