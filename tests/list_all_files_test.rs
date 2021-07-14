use raze::api::*;
use raze::utils::*;
mod common;
use common::*;

#[tokio::test]
async fn test_list_all_files() {
    use futures::StreamExt;
    use futures::TryStreamExt;
    let TestSetup {
        client,
        auth,
        bucket_id,
    } = setup_test_with_auth().await;

    let expected_files = b2_list_file_names(&client, &auth, &bucket_id, "", 16)
        .await
        .unwrap()
        .files;

    let stream = list_all_files_stream(client, auth, bucket_id, 4);
    let files: Vec<B2FileInfo> = stream.take(16).try_collect().await.unwrap();

    assert_eq!(files, expected_files);
}
