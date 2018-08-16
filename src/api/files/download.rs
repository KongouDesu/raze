use reqwest;
use B2Error;
use serde_json;
use api::auth::B2Auth;

use api::files::structs::*;

/// Download a file by it's assigned id
///
/// Returns a struct containing the file info and a string containing the raw file
///
/// Equivalent to b2_download_file_by_id - [Official documentation](https://www.backblaze.com/b2/docs/b2_download_file_by_id.html)
pub fn download_file_by_id(client: &reqwest::Client, auth: &B2Auth, file_id: &str) -> Result<(FileInfo,String), B2Error> {
    let mut headers = reqwest::header::Headers::new();
    // The auth token from an UploadAuth, see: get_upload_url
    headers.set(reqwest::header::Authorization(auth.authorization_token.clone()));
    // Send the request
    let mut resp = match client.get(&format!("{}/b2api/v1/b2_download_file_by_id?fileId={}",auth.download_url,file_id))
        .headers(headers)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Test if the response body is an error, if not, the body will be the file and we continue to extract headers
    use B2ApiError;
    use B2Error;
    match serde_json::from_str::<B2ApiError>(&response_string) {
        Ok(v) => return Err(B2Error::B2Error(v)),
        Err(_e) => (),
    };
    let headers = resp.headers();
    use reqwest::header::*;
    let len = match headers.get::<ContentLength>() {
        Some(t) => Some(t.0),
        None => None,
    };
    let content_type = match headers.get::<ContentType>() {
        Some(t) => Some(format!("{}",t)),
        None => None,
    };
    let fileinfo = FileInfo {
        content_length: len,
        content_type,
        x_bz_content_sha1: raw_header_to_str(headers,"x-bz-content-sha1"),
        x_bz_file_id: raw_header_to_str(headers,"x-bz-file-id"),
        x_bz_file_name: raw_header_to_str(headers,"x-bz-file-name"),
        x_bz_info_src_last_modified_millis: raw_header_to_u64(headers,"x-bz-info-src_last_modified_millis"),
        x_bz_upload_timestamp: raw_header_to_u64(headers, "X-Bz-Upload-Timestamp"),
    };
    Ok((fileinfo, response_string))
}

/// Download a file by it's given name
///
/// Returns a struct containing the file info and a string containing the raw file
///
/// The file name should be the full name on the server, eg. if you're emulating folders you should use the full path
///
/// Equivalent to b2_download_file_by_name - [Official documentation](https://www.backblaze.com/b2/docs/b2_download_file_by_name.html)
pub fn download_file_by_name(client: &reqwest::Client, auth: &B2Auth, bucket_name: &str, file_name: &str) -> Result<(FileInfo,String), B2Error> {
    let mut headers = reqwest::header::Headers::new();
    // The auth token from an UploadAuth, see: get_upload_url
    headers.set(reqwest::header::Authorization(auth.authorization_token.clone()));
    // Send the request
    let mut resp = match client.get(&format!("{}/file/{}/{}",auth.download_url,bucket_name, file_name))
        .headers(headers)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Test if the response body is an error, if not, the body will be the file and we continue to extract headers
    use B2ApiError;
    use B2Error;
    match serde_json::from_str::<B2ApiError>(&response_string) {
        Ok(v) => return Err(B2Error::B2Error(v)),
        Err(_e) => (),
    };
    let headers = resp.headers();
    use reqwest::header::*;
    let len = match headers.get::<ContentLength>() {
        Some(t) => Some(t.0),
        None => None,
    };
    let content_type = match headers.get::<ContentType>() {
        Some(t) => Some(format!("{}",t)),
        None => None,
    };
    let fileinfo = FileInfo {
        content_length: len,
        content_type,
        x_bz_content_sha1: raw_header_to_str(headers,"x-bz-content-sha1"),
        x_bz_file_id: raw_header_to_str(headers,"x-bz-file-id"),
        x_bz_file_name: raw_header_to_str(headers,"x-bz-file-name"),
        x_bz_info_src_last_modified_millis: raw_header_to_u64(headers,"x-bz-info-src_last_modified_millis"),
        x_bz_upload_timestamp: raw_header_to_u64(headers, "X-Bz-Upload-Timestamp"),
    };
    Ok((fileinfo, response_string))
}

/// Helper method for extracting headers from a 'Headers' struct
///
/// The raw headers are &[u8], we want a String
/// Returns a String if the header exists, None if it doesn't
/// # Panics
/// Panics if the header is invalid utf-8
fn raw_header_to_str(headers: &reqwest::header::Headers, header: &str) -> Option<String> {
    use std::str;
    match headers.get_raw(header) {
        Some(raw) => {
            match raw.one() {
                Some(bytes) => Some(str::from_utf8(bytes).to_owned().unwrap().to_owned()),
                None => None,
            }
        },
        None => None,
    }
}

/// Helper method for extracting headers from a 'Headers' struct
///
/// The raw headers are &[u8], in this case we want a u64
/// Returns a u64 if the header exists, None if it doesn't
fn raw_header_to_u64(headers: &reqwest::header::Headers, header: &str) -> Option<u64> {
    use std::u64;
    use std::str::FromStr;
    let as_str = raw_header_to_str(headers, header);
    match as_str {
        Some(s) => Some(u64::from_str(&s).unwrap()),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use api::auth;
    use std;
    use reqwest;
    use api::files::*;
    use api::files::upload::*;
    use ::tests::TEST_CREDENTIALS_FILE;
    use ::tests::TEST_BUCKET_ID;

    // Tests that we can upload a file, fails if get_upload_url fails
    // Tests that we can use both download methods
    // Tests that we can delete a file, won't be tested if uploading fails
    #[test]
    #[allow(unused_variables)]
    fn test_download_file() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = misc::get_upload_url(&client, &autho, TEST_BUCKET_ID).unwrap();
        let n = upload_file(&client, &res, std::path::Path::new("./testfile.txt"), "some_folder").unwrap();

        let by_id = download::download_file_by_id(&client, &autho, &n.file_id);
        let by_name = download::download_file_by_name(&client, &autho, "uploaded-files", "some_folder/testfile.txt");
        // Delete file before assertions to prevent leaving files around
        let x = misc::delete_file_version(&client, &autho, n.file_name, n.file_id);

        match by_id {
            Ok(_t) => (),
            Err(_e) => assert!(false),
        }

        match by_name {
            Ok(_t) => (),
            Err(_e) => assert!(false),
        }
    }
}