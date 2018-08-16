use reqwest;
use api::auth;
use B2Error;
use serde_json;

use handle_b2error_kinds;
use api::files::structs::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// Request body used for the [b2_get_upload_url](https://www.backblaze.com/b2/docs/b2_get_upload_url.html) call
struct GetUploadUrlBody<'a> {
    bucket_id: &'a str,
}

/// Gets an upload url and an auth token for the bucket with the supplied id
///
/// Files are uploaded to the url and use the token to authenticate
///
/// Official documentation: [b2_get_upload_url](https://www.backblaze.com/b2/docs/b2_get_upload_url.html)
pub fn get_upload_url(client: &reqwest::Client, auth: &auth::B2Auth, bucket_id: &str) -> Result<UploadAuth, B2Error> {
    // Set auth header
    let header = reqwest::header::Authorization(auth.authorization_token.clone());
    // Turn our request into JSON
    let json = match serde_json::to_string(&GetUploadUrlBody {
        bucket_id: &bucket_id,
    }) {
        Ok(v) => v,
        Err(e) => return Err(B2Error::SerdeError(e))
    };
    // Set the body and send the request
    let body = reqwest::Body::from(json);
    let mut resp = match client.post(&format!("{}{}", auth.api_url, "/b2api/v1/b2_get_upload_url"))
        .header(header)
        .body(body)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: UploadAuth = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

// Used for BOTH sending AND receiving info about deleting files
#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
/// Request body and returned result from the [b2_delete_file_version](https://www.backblaze.com/b2/docs/b2_delete_file_version.html) API call
pub struct DeleteFile {
    file_name: String,
    file_id: String
}

/// Deletes a specific version of the file with the supplied name
///
/// The file_name is the full filename as stored on the server. Remember that BackBlaze does not have folders
///
/// Official documentation: [b2_delete_file_version](https://www.backblaze.com/b2/docs/b2_delete_file_version.html)
pub fn delete_file_version(client: &reqwest::Client, auth: &auth::B2Auth, file_name: String, file_id: String) -> Result<DeleteFile,B2Error> {
    // Auth token header
    let header = reqwest::header::Authorization(auth.authorization_token.clone());
    // Create the JSON body
    let json = match serde_json::to_string(&DeleteFile {
        file_name,
        file_id
    }) {
        Ok(v) => v,
        Err(e) => return Err(B2Error::SerdeError(e))
    };
    // Set the body to the created JSON
    let body = reqwest::Body::from(json);

    // Send the request
    let mut resp = match client.post(&format!("{}{}", auth.api_url, "/b2api/v1/b2_delete_file_version"))
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
    let deserialized: DeleteFile = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// A list of [stored files](../structs/struct.StoredFile.html), obtained via [list_file_names](fn.list_file_names.html)
///
/// The BackBlaze API returns a maximum of 10000 files per call \
/// The 'next_file_name' can be used to pick up where the previous call left off
///
/// See [list_file_names](fn.list_file_names.html)
pub struct FileList {
    pub files: Vec<StoredFile>,
    pub next_file_name: Option<String>,
}

/// Returns a list of up to 'max_file_count' files stored on the server
///
/// A single call will return at most max_file_count results, if there's more files, a start_file_name can be supplied \
/// If start_file_name is set, it will list beginning from said file name \
/// Setting max_file_count to 0 will use the default value of 100
///
/// max_file_count can be at most 10000 per call. Note that this is a class C transaction per started 1000 returned files
///
/// Official documentation: [b2_list_file_names](https://www.backblaze.com/b2/docs/b2_list_file_names.html)
pub fn list_file_names(client: &reqwest::Client, auth: &auth::B2Auth, bucket_id: &str, start_file_name: &str, max_file_count: usize) -> Result<FileList,B2Error> {
    // Auth token header
    let header = reqwest::header::Authorization(auth.authorization_token.clone());
    // Create the JSON body
    let json = match serde_json::to_string(&ListFileBody {
        bucket_id: bucket_id.to_owned(),
        start_file_name: start_file_name.to_owned(),
        max_file_count,
    }) {
        Ok(v) => v,
        Err(e) => return Err(B2Error::SerdeError(e))
    };
    // Set the body to the created JSON
    let body = reqwest::Body::from(json);

    // Send the request
    let mut resp = match client.post(&format!("{}{}", auth.api_url, "/b2api/v1/b2_list_file_names"))
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
    let deserialized: FileList = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

/// Just like [list_file_names](fn.list_file_names.html) but will repeat until all "file names" have been retrieved
///
/// Will request 'max_file_count' files at a time, setting this low may result in a large amount of transactions
/// Returns a Vec<StoredFile> since it's guaranteed there will be no 'next_file_name'
///
/// Official documentation: [b2_list_file_names](https://www.backblaze.com/b2/docs/b2_list_file_names.html)
pub fn list_all_file_names(client: &reqwest::Client, auth: &auth::B2Auth, bucket_id: &str, max_file_count: usize) -> Result<Vec<StoredFile>,B2Error> {
    let mut list = match list_file_names(&client, &auth, &bucket_id, &"", max_file_count) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    while list.next_file_name.is_some() {
        let mut next_list = match list_file_names(&client, &auth, &bucket_id, &list.next_file_name.unwrap(), max_file_count) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        list.files.append(&mut next_list.files);
        list.next_file_name = next_list.next_file_name;
    }
    Ok(list.files)
}




#[cfg(test)]
mod tests {
    use api::auth;
    use std;
    use reqwest;
    use api::files::*;
    use ::tests::TEST_CREDENTIALS_FILE;
    use ::tests::TEST_BUCKET_ID;

    // Tests that we can get an upload url
    // No need to verify the returned value, as it's guaranteed to be a valid UploadAuth
    #[test]
    #[allow(unused_variables)]
    fn test_get_upload_url() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = misc::get_upload_url(&client, &autho, TEST_BUCKET_ID).unwrap();
    }

    // list_all_file_names calls list_file_names, so this tests both
    // Only needs to not panic to pass
    #[test]
    #[allow(unused_variables)]
    fn test_list_all_file_names() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = misc::list_all_file_names(&client, &autho, TEST_BUCKET_ID, 1000);
    }
}