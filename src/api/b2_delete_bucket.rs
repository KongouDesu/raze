use crate::api::{B2Auth, BucketResult};
use crate::handle_b2error_kinds;
use crate::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DeleteBucketBody<'a> {
    account_id: &'a str,
    bucket_id: &'a str,
}

/// <https://www.backblaze.com/b2/docs/b2_delete_bucket.html>
pub async fn b2_delete_bucket<T: AsRef<str>>(
    client: &Client,
    auth: &B2Auth,
    bucket_id: T,
) -> Result<BucketResult, Error> {
    let req_body = serde_json::to_string(&DeleteBucketBody {
        account_id: &auth.account_id,
        bucket_id: bucket_id.as_ref(),
    })
    .unwrap();

    let resp = match client
        .post(&auth.api_url_for("b2_delete_bucket"))
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
    let deserialized: BucketResult = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string));
        }
    };
    Ok(deserialized)
}
