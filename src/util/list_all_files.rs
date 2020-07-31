use reqwest::blocking::Client;
use api::{B2Auth, ListFilesResult};
use Error;
use api;

/// A version of b2_list_file_names that loops until all files have been retrieved
/// Note billing behavior regarding 'max_file_count', recommended value is 1000
/// https://www.backblaze.com/b2/docs/b2_list_file_names.html
pub fn list_all_files<T: AsRef<str>>(client: &Client, auth: &B2Auth, bucket_id: T, max_file_count: u32) -> Result<ListFilesResult, Error> {
    let mut list = match api::b2_list_file_names(&client, &auth, &bucket_id, &"", max_file_count) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    while list.next_file_name.is_some() {
        let mut next_list = match api::b2_list_file_names(&client, &auth, &bucket_id, &list.next_file_name.unwrap(), max_file_count) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        list.files.append(&mut next_list.files);
        list.next_file_name = next_list.next_file_name;
    }

    Ok(list)
}