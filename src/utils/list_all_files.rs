use std::borrow::Cow;
use std::collections::VecDeque;

use crate::api::{b2_list_file_names, ListFilesResult};
use crate::api::{B2Auth, B2FileInfo};
use crate::Error;
use futures::Stream;
use reqwest::Client;

/// Get a stream of all file infos in the bucket using [b2_list_file_names]
///
/// Lazily calls the API as the stream is consumed. \
/// The recommended value for `batch_size` is the maximum value possible: 1000.
///
/// <https://www.backblaze.com/b2/docs/b2_list_file_names.html>
pub fn list_all_files_stream<T: Into<Cow<'static, str>>>(
    client: Client,
    auth: B2Auth,
    bucket_id: T,
    batch_size: u32,
) -> impl Stream<Item = Result<B2FileInfo, Error>> {
    struct ListAllFilesSeed {
        client: Client,
        auth: B2Auth,
        bucket_id: Cow<'static, str>,
        batch_size: u32,
        next_file_name: Option<Cow<'static, str>>,
        batch: VecDeque<B2FileInfo>,
    }
    async fn inner(
        mut seed: ListAllFilesSeed,
    ) -> Option<(Result<B2FileInfo, Error>, ListAllFilesSeed)> {
        if let Some(front) = seed.batch.pop_front() {
            Some((Ok(front), seed))
        } else {
            if let Some(file_name_str) = &seed.next_file_name {
                let res = b2_list_file_names(
                    &seed.client,
                    &seed.auth,
                    &seed.bucket_id,
                    file_name_str,
                    seed.batch_size,
                )
                .await;
                match res {
                    Ok(ListFilesResult {
                        files,
                        next_file_name,
                    }) => {
                        let mut iter = files.into_iter();
                        let front = iter.next();
                        seed.batch.extend(iter);
                        seed.next_file_name = next_file_name.map(Cow::from);
                        if let Some(front) = front {
                            Some((Ok(front), seed))
                        } else {
                            None
                        }
                    }
                    Err(err) => Some((Err(err), seed)),
                }
            } else {
                None
            }
        }
    }
    futures::stream::unfold(
        ListAllFilesSeed {
            client,
            auth,
            bucket_id: bucket_id.into(),
            batch_size,
            next_file_name: Some("".into()),
            batch: VecDeque::new(),
        },
        inner,
    )
}
