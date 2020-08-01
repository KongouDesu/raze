use reqwest::blocking::Client;
use crate::api::{B2Auth, B2FileInfo};
use crate::Error;
use crate::api;

/// List *all* files inside a given bucket by repeating [b2_list_file_names](../api/fn.b2_list_file_names.html)
///
/// Keeps calling the API until all files have been retrieved \
/// Note billing behavior regarding 'max_file_count', recommended value is 1000 \
/// <https://www.backblaze.com/b2/docs/b2_list_file_names.html>
pub fn list_all_files<T: AsRef<str>>(client: &Client, auth: &B2Auth, bucket_id: T, max_file_count: u32) -> Result<Vec<B2FileInfo>, Error> {
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

    Ok(list.files)
}