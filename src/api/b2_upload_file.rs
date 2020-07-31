use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use ::Error;
use api::{UploadAuth, B2FileInfo};
use handle_b2error_kinds;
use std::io::Read;

#[derive(Debug)]
pub struct FileParameters<'a> {
    pub file_path: &'a str,
    pub file_size: u64,
    pub content_type: Option<&'a str>,
    pub content_sha1: Sha1Variant<'a>,
    pub last_modified_millis: u64,
}

/// The different types of Sha1-checksums supported
/// Precomputed requires the hash computed before you start the upload
/// HexAtEnd expects the 'file' Reader to provide the Sha1 as 40-characters hexadecimal at the end
/// DoNotVerify will use no hash at all. Note that this is not recommended by Backblaze
#[derive(Debug)]
pub enum Sha1Variant<'a> {
    Precomputed(&'a str),
    HexAtEnd,
    DoNotVerify,
}

/// https://www.backblaze.com/b2/docs/b2_upload_file.html
/// If 'content_type' is None, "b2/x-auto" is used as default
/// Be aware of Sha1-checksum behavior
pub fn b2_upload_file<R: 'static + Read + Send>(client: &Client, auth: &UploadAuth, file: R, params: FileParameters) -> Result<B2FileInfo, Error> {
    let mut headers = HeaderMap::new();
    // Encode the file name
    // See https://www.backblaze.com/b2/docs/string_encoding.html
    // Note we need to drop the first character, as it is always an equals '=' symbol
    let encoded_file_name = &url::form_urlencoded::Serializer::new(String::new())
        .append_pair("",params.file_path)
        .finish()[1..];

    let hash = match params.content_sha1 {
        Sha1Variant::Precomputed(hash) => hash,
        Sha1Variant::HexAtEnd => "hex_digits_at_end",
        Sha1Variant::DoNotVerify => "do_not_verify",
    };

    // If we use hex digits at end, we need to add 40 bytes to account for the hex characters
    let file_size = match params.content_sha1 {
        Sha1Variant::HexAtEnd => params.file_size + 40,
        _ => params.file_size,
    };


    headers.insert(reqwest::header::AUTHORIZATION, auth.authorization_token.parse().unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE,params.content_type.unwrap_or("b2/x-auto").parse().unwrap());
    headers.insert(reqwest::header::CONTENT_LENGTH,file_size.into());
    headers.insert("X-Bz-File-Name",(&encoded_file_name).parse().unwrap());
    headers.insert("X-Bz-Content-Sha1",hash.parse().unwrap());
    headers.insert("X-Bz-Info-src_last_modified_millis",params.last_modified_millis.into());

    let body = reqwest::blocking::Body::sized(file, file_size);

    let resp = match client.post(&auth.upload_url)
        .headers(headers)
        .body(body)
        .send() {
        Ok(v) => v,
        Err(e) => return Err(Error::ReqwestError(e))
    };
    if !resp.status().is_success() {
        return Err(Error::from_response(resp))
    }

    let response_string = resp.text().unwrap();
    let deserialized: B2FileInfo = match serde_json::from_str(&response_string) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("{:?}", response_string);
            return Err(handle_b2error_kinds(&response_string))
        }
    };
    Ok(deserialized)
}