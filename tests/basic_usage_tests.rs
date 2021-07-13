use raze::api::*;
use raze::util::*;
mod common;
use common::*;

#[tokio::test]
async fn test_basic_usage() {
    let TestSetup {
        client,
        auth,
        bucket_id,
    } = setup_test_with_auth().await;

    let upauth = b2_get_upload_url(&client, &auth, &bucket_id).await.unwrap();

    let file = open_test_file("simple_text_file.txt").await;
    let metadata = file.metadata().await.unwrap();
    let size = metadata.len();
    let modf = metadata
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        * 1000;

    let param = FileParameters {
        file_path: "simple_text_file.txt",
        file_size: size,
        content_type: None,
        content_sha1: Sha1Variant::HexAtEnd,
        last_modified_millis: modf,
    };

    let reader = file;
    let reader = AsyncReadHashAtEnd::wrap(reader);
    let reader = AsyncReadThrottled::wrap(reader, 20000);

    let resp1 = b2_upload_file(&client, &upauth, body_from_reader(reader), param).await;

    let resp1 = resp1.unwrap();
    assert_eq!(
        resp1
            .file_info
            .unwrap()
            .get("src_last_modified_millis")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        modf
    );

    let info = b2_get_file_info(&client, &auth, resp1.file_id.as_ref().unwrap())
        .await
        .unwrap();
    assert_eq!(info.modified(), modf);

    let param2 = B2GetDownloadAuthParams {
        bucket_id: bucket_id.to_string(),
        file_name_prefix: "".to_string(),
        valid_duration_in_seconds: 500,
    };

    b2_get_download_authorization(&client, &auth, param2)
        .await
        .unwrap();

    b2_delete_file_version(&client, &auth, &resp1.file_name, &resp1.file_id.unwrap())
        .await
        .unwrap();
}
