use crate::api::{B2Auth, B2DownloadAuth};
use crate::Error;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Parameters for b2_download_file_by_name
///
/// Note that authorization is only required if you want to make use of the prefix and/or expiration offered by b2_get_download_authorization
/// If authorization is None, the B2Auth is used instead
pub struct B2DownloadFileByNameParams {
    pub bucket_name: String,
    pub file_name: String,
    pub authorization: Option<B2DownloadAuth>,
}

/// <https://www.backblaze.com/b2/docs/b2_download_file_by_name.html>
pub async fn b2_download_file_by_name(
    client: &Client,
    auth: &B2Auth,
    params: B2DownloadFileByNameParams,
) -> Result<Response, Error> {
    let auth_token = match params.authorization {
        Some(ref a) => &a.authorization_token,
        None => &auth.authorization_token,
    };

    let resp = match client
        .get(&auth.download_url_by_name(&params.bucket_name, &params.file_name))
        .header(reqwest::header::AUTHORIZATION, auth_token)
        .send()
        .await
    {
        Ok(v) => v,
        Err(e) => return Err(Error::ReqwestError(e)),
    };
    if !resp.status().is_success() {
        return Err(Error::from_response(resp).await);
    }

    Ok(resp)
}
