use reqwest;
use api::auth;
use B2Error;
use serde_json;
use handle_b2error_kinds;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// A bucket
///
/// Note that bucket_info and lifecycle_rules are currently not supported
pub struct Bucket {
    pub account_id: String,
    pub bucket_id: String,
    //pub bucket_info: type,
    pub bucket_name: String,
    pub bucket_type: BucketType,
    //pub lifecycle_rules: type,
    pub revision: u32
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
struct BucketList {
    buckets: Vec<Bucket>
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
/// The type of bucket, per BackBlaze API
///
/// For more info, see [b2_update_bucket](https://www.backblaze.com/b2/docs/b2_update_bucket.html)
pub enum BucketType {
    AllPublic,
    AllPrivate,
    Snapshot
}

/// Gets a list of buckets for the account associated with the auth
///
/// Official documentation: [b2_list_buckets](https://www.backblaze.com/b2/docs/b2_list_buckets.html)
pub fn list_buckets(client: &reqwest::Client, auth: &auth::B2Auth) -> Result<Vec<Bucket>, B2Error>{
    // The auth token as header
    let header = reqwest::header::Authorization(auth.authorization_token.clone());
    // JSON eg. {"accountId": "xxxxxxxxxxxx"}, accountId is taken from supplied auth
    let body = reqwest::Body::from(format!("{}{}{}","{\"accountId\": \"", auth.account_id, "\"}"));
    let mut resp = match client.post(&format!("{}{}", auth.api_url, "/b2api/v1/b2_list_buckets"))
        .header(header)
        .body(body)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    if resp.status().is_client_error() {
        return Err(B2Error::from_response(resp))
    }
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: BucketList = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized.buckets)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateBucketBody<'a> {
    account_id: &'a str,
    bucket_name: &'a str,
    bucket_type: BucketType,
}

/// Create a bucket on the account associated with the auth with the given name and type
///
/// Official documentation: [b2_create_bucket](https://www.backblaze.com/b2/docs/b2_create_bucket.html)
pub fn create_bucket(client: &reqwest::Client, auth: &auth::B2Auth, name: &str, bucket_type: BucketType) -> Result<Bucket, B2Error> {
    // Set auth header
    let header = reqwest::header::Authorization(auth.authorization_token.clone());
    // Turn our request into JSON
    let json = match serde_json::to_string(&CreateBucketBody {
        account_id: &auth.account_id,
        bucket_name: name,
        bucket_type: bucket_type
    }) {
        Ok(v) => v,
        Err(e) => return Err(B2Error::SerdeError(e))
    };
    // Set the body and send the request
    let body = reqwest::Body::from(json);
    let mut resp = match client.post(&format!("{}{}", auth.api_url, "/b2api/v1/b2_create_bucket"))
        .header(header)
        .body(body)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: Bucket = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeleteBucketBody<'a> {
    account_id: &'a str,
    bucket_id: &'a str,
}

/// Delete a bucket on the account associated with the auth with the given id
///
/// Official documentation: [b2_delete_bucket](https://www.backblaze.com/b2/docs/b2_delete_bucket.html)
///
/// Returns the deleted bucket upon success
pub fn delete_bucket(client: &reqwest::Client, auth: &auth::B2Auth, id: &str) -> Result<Bucket, B2Error> {
    // Set auth header
    let header = reqwest::header::Authorization(auth.authorization_token.clone());
    // Turn our request into JSON
    let json = match serde_json::to_string(&DeleteBucketBody {
        account_id: &auth.account_id,
        bucket_id: id
    }) {
        Ok(v) => v,
        Err(e) => return Err(B2Error::SerdeError(e))
    };
    // Set the body and send the request
    let body = reqwest::Body::from(json);
    let mut resp = match client.post(&format!("{}{}", auth.api_url, "/b2api/v1/b2_delete_bucket"))
        .header(header)
        .body(body)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: Bucket = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

/// Updates the settings of a bucket, including type, info, CORS and lifecycle rules \
/// NOT IMPLEMENTED YET
///
/// A revision number can be providing, preventing updates unless the provided and stored value match
/// Official documentation: [b2_update_bucket](https://www.backblaze.com/b2/docs/b2_update_bucket.html)
pub fn update_bucket(){
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use api::auth;
    use std;
    use reqwest;
    use api::buckets;
    use ::tests::TEST_CREDENTIALS_FILE as TEST_CREDENTIALS_FILE;

    // This also tests 'authenticate', as 'authenticate_from_file' calls it
    // The test passes as long as the call doesn't fail
    #[test]
    #[allow(unused_variables)]
    fn test_list_buckets() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = buckets::list_buckets(&client, &autho).unwrap();
    }
}