use reqwest;
use B2Error;
use serde_json;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std;
use sha1;
use std::io::BufReader;
use url::form_urlencoded;

use handle_b2error_kinds;
use api::files::structs::*;

// Define custom headers used for uploading files
header! {
    /// File name header as per [B2 API docs](https://www.backblaze.com/b2/docs/b2_upload_file.html)
    (XBzFileName, "X-Bz-File-Name") => [String]
}
header! {
    /// MIME Content Type header
    (ContentType, "Content-Type") => [String]
}
header! {
    /// Header containing the Sha1-hash of the uploaded file
    (XBzContentsha1, "X-Bz-Content-Sha1") => [String]
}
header! {
    /// The file's last modified time in milliseconds since unix epoch. [See documentation](https://www.backblaze.com/b2/docs/b2_upload_file.html)
    (XBzInfoSrcLastModifiedMillis, "X-Bz-Info-src_last_modified_millis") => [u64]
}

/// Upload a file as to the bucket associated with the UploadAuth
/// Equivalent to b2_upload_file - [Official documentation](https://www.backblaze.com/b2/docs/b2_upload_file.html)
///
/// The upload filed will have the name of the file provided via the path and the prefix like \
/// prefix + file_name - For example "pictures/" + "dog.jpg" is stored as "pictures/dog.jpg"
///
/// You are responsible for setting the prefix if you want to emulate folders
///
/// This method will load the entire file into memory, hash it and send the hash in the header \
/// It also automatically formats the file name, sets the MIME type and modified date
/// # Errors
/// See [official documentation](https://www.backblaze.com/b2/docs/b2_upload_file.html) section on errors
pub fn upload_file(client: &reqwest::Client, upload_auth: &UploadAuth, file_path: &Path, prefix: &str) -> Result<StoredFile, B2Error> {
    let file = File::open(file_path).unwrap();
    let metadata = std::fs::metadata(file_path).unwrap();
    let mut bytes = Vec::new();
    let mut reader = BufReader::new(file);
    reader.read_to_end(&mut bytes).unwrap();
    let file_hash = sha1::Sha1::from(&bytes).hexdigest();
    // We need to set a variety of headers to upload a file
    let mut headers = reqwest::header::Headers::new();
    // The auth token from an UploadAuth, see: get_upload_url
    headers.set(reqwest::header::Authorization(upload_auth.authorization_token.clone()));

    // Apply the prefix only if it isn't blank
    let file_name: String;
    match prefix {
        "" => file_name = file_path.file_name().unwrap().to_str().unwrap().to_owned(),
        _ => file_name = format!("{}/{}", prefix, file_path.file_name().unwrap().to_str().unwrap()),
    }
    let file_name = file_name.replace("\\", "/");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("", &file_name)
        .finish();
    // The first character of the 'encoded' is always '=', which we don't need
    // This is a side effect of the way the serializer works
    let percent_encoded = &encoded[1..];
    headers.set(XBzFileName(percent_encoded.to_string()));
    // The MIME type. b2/x-auto will have B2 automatically set it based on file extension
    // If no extension is present, defaults to application/octet-stream
    headers.set(ContentType("b2/x-auto".to_owned()));
    // The amount of bytes being uploaded PLUS 40 for the SHA1
    headers.set(reqwest::header::ContentLength(bytes.len() as u64));
    // Tell it the SHA1 is at the end of the request body
    headers.set(XBzContentsha1(file_hash));
    // Set the last-modified-date of the file
    // Get the last modified time in unix epoch time, convert to a single number representing this in millis
    // Note that this intentionally has a resolution of 1 second. The API expects millis, so convert s -> ms
    let modified_time = match metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH) {
        Ok(v) => v.as_secs()*1000,
        Err(_e) => 0u64
    };
    headers.set(XBzInfoSrcLastModifiedMillis(modified_time));
    // Send the request
    let mut resp = match client.post(&upload_auth.upload_url)
        .headers(headers)
        .body(bytes)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: StoredFile = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

/// Upload the file using a streaming read
///
/// This will send the hash at the end of the body instead of as a header \
/// This is slower than upload_file, but uses significantly less memory, especially for large files
///
/// For full documentation, see [upload_file](fn.upload_file.html)
pub fn upload_file_streaming(client: &reqwest::Client, upload_auth: &UploadAuth, file_path: &Path, prefix: &str) -> Result<StoredFile, B2Error> {
    let file = File::open(file_path).unwrap();
    let metadata = std::fs::metadata(file_path).unwrap();

    // We need to set a variety of headers to upload a file
    // NOTICE: Unlike upload_file, we do not have to set the Content-Length header ourselves
    let mut headers = reqwest::header::Headers::new();
    // The auth token from an UploadAuth, see: get_upload_url
    headers.set(reqwest::header::Authorization(upload_auth.authorization_token.clone()));

    // Apply the prefix only if it isn't blank
    let file_name: String;
    match prefix {
        "" => file_name = file_path.file_name().unwrap().to_str().unwrap().to_owned(),
        _ => file_name = format!("{}/{}", prefix, file_path.file_name().unwrap().to_str().unwrap()),
    }
    let file_name = file_name.replace("\\", "/");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("", &file_name)
        .finish();
    // The first character of the 'encoded' is always '=', which we don't need
    // This is a side effect of the way the serializer works
    let percent_encoded = &encoded[1..];
    headers.set(XBzFileName(percent_encoded.to_string()));
    // The MIME type. b2/x-auto will have B2 automatically set it based on file extension
    // If no extension is present, defaults to application/octet-stream
    headers.set(ContentType("b2/x-auto".to_owned()));
    // Tell it the SHA1 is at the end of the request body
    headers.set(XBzContentsha1("hex_digits_at_end".to_owned()));
    // Set the last-modified-date of the file
    // Get the last modified time in unix epoch time, convert to a single number representing this in millis
    // Note that this intentionally has a resolution of 1 second. The API expects millis, so convert s -> ms
    let modified_time = match metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH) {
        Ok(v) => v.as_secs()*1000,
        Err(_e) => 0u64
    };
    headers.set(XBzInfoSrcLastModifiedMillis(modified_time));
    // Send the request
    let mut resp = match client.post(&upload_auth.upload_url)
        .headers(headers)
        .body(reqwest::Body::sized(HashAtEndOfBody::new(file), metadata.len()+40))
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: StoredFile = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

/// Like [upload_file_streaming](fn.upload_file_streaming.html) but allows you to limit the upload bandwidth
///
/// bandwidth is the maximum amount of bytes per second
///
/// Note: this is slower than the other upload methods, regardless of throttling, due to overhead
///
/// For full documentation, see [upload_file](fn.upload_file.html) \
/// Official documentation: [b2_upload_file](https://www.backblaze.com/b2/docs/b2_upload_file.html)
pub fn upload_file_throttled(client: &reqwest::Client, upload_auth: &UploadAuth, file_path: &Path, prefix: &str, bandwidth: usize) -> Result<StoredFile, B2Error> {
    let file = File::open(file_path).unwrap();
    let metadata = std::fs::metadata(file_path).unwrap();

    // We need to set a variety of headers to upload a file
    // NOTICE: Unlike upload_file, we do not have to set the Content-Length header ourselves
    let mut headers = reqwest::header::Headers::new();
    // The auth token from an UploadAuth, see: get_upload_url
    headers.set(reqwest::header::Authorization(upload_auth.authorization_token.clone()));

    // Apply the prefix only if it isn't blank
    let file_name: String;
    match prefix {
        "" => file_name = file_path.file_name().unwrap().to_str().unwrap().to_owned(),
        _ => file_name = format!("{}/{}", prefix, file_path.file_name().unwrap().to_str().unwrap()),
    }
    let file_name = file_name.replace("\\", "/");
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("", &file_name)
        .finish();
    // The first character of the 'encoded' is always '=', which we don't need
    // This is a side effect of the way the serializer works
    let percent_encoded = &encoded[1..];
    headers.set(XBzFileName(percent_encoded.to_string()));
    // The MIME type. b2/x-auto will have B2 automatically set it based on file extension
    // If no extension is present, defaults to application/octet-stream
    headers.set(ContentType("b2/x-auto".to_owned()));
    // Tell it the SHA1 is at the end of the request body
    headers.set(XBzContentsha1("hex_digits_at_end".to_owned()));
    // Set the last-modified-date of the file
    // Get the last modified time in unix epoch time, convert to a single number representing this in millis
    // Note that this intentionally has a resolution of 1 second. The API expects millis, so convert s -> ms
    let modified_time = match metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH) {
        Ok(v) => v.as_secs()*1000,
        Err(_e) => 0u64
    };
    headers.set(XBzInfoSrcLastModifiedMillis(modified_time));
    // Send the request
    let mut resp = match client.post(&upload_auth.upload_url)
        .headers(headers)
        .body(reqwest::Body::sized(ThrottledHashAtEndOfBody::new(file, bandwidth), metadata.len()+40))
        .send() {
        Ok(v) => v,
        Err(e) => return Err(B2Error::ReqwestError(e))
    };
    // Read the response to a string containing the JSON response
    let response_string = resp.text().unwrap();
    // Convert the response string from JSON to a struct
    let deserialized: StoredFile = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}

#[cfg(test)]
mod tests {
    use api::auth;
    use std;
    use reqwest;
    use api::files::*;
    use api::files::upload::*;
    use api::files::misc::*;
    use ::tests::TEST_CREDENTIALS_FILE;
    use ::tests::TEST_BUCKET_ID;

    // Tests that we can upload a file, fails if get_upload_url fails
    // Tests that we can delete a file, won't be tested if uploading fails
    // TODO This leaves a mess if upload works but delete doesn't
    #[test]
    #[allow(unused_variables)]
    fn test_upload_and_delete_file() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = misc::get_upload_url(&client, &autho, TEST_BUCKET_ID).unwrap();
        let n = upload_file(&client, &res, std::path::Path::new("./testfile.txt"), "some_folder").unwrap();
        let x = misc::delete_file_version(&client, &autho, n.file_name, n.file_id).unwrap();
    }

    // Test upload_file_streaming
    #[test]
    #[allow(unused_variables)]
    fn test_upload_file_streaming() {
        let client = reqwest::Client::new();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = get_upload_url(&client, &autho, TEST_BUCKET_ID).unwrap();
        let n = upload_file_streaming(&client, &res, std::path::Path::new("./testblob.txt"), "some_folder").unwrap();
        let x = misc::delete_file_version(&client, &autho, n.file_name, n.file_id).unwrap();
    }

    // Test upload_file_throttled
    #[test]
    #[allow(unused_variables)]
    fn test_upload_file_throttled() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::new(100000000,0))
            .build()
            .unwrap();
        let autho = auth::authenticate_from_file(&client, std::path::Path::new(TEST_CREDENTIALS_FILE)).unwrap();
        let res = get_upload_url(&client, &autho, TEST_BUCKET_ID).unwrap();

        let dur = std::time::SystemTime::now();

        // Bandwidth is one tenth of the testblob_small.txt size, as thus, this upload can't be faster than 10s
        let n = upload_file_throttled(&client, &res, std::path::Path::new("./testblob_small.txt"), "some_folder", 11500).unwrap();
        //println!("Upload took {} seconds and {} ns",dur.elapsed().unwrap().as_secs(), dur.elapsed().unwrap().subsec_nanos());
        let time_elapsed = dur.elapsed().unwrap().as_secs() as f64 + dur.elapsed().unwrap().subsec_nanos() as f64/1000000000.;
        let x = misc::delete_file_version(&client, &autho, n.file_name, n.file_id).unwrap();
        assert!(time_elapsed > 10., format!("Upload was faster than throttle allows! ({} seconds)",time_elapsed))
    }
}