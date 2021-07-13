use raze::api::{self, B2Auth};
use reqwest::Client;
use tokio::{fs::File, sync::OnceCell};

pub struct TestSetup {
    pub client: Client,
    pub auth: B2Auth,
    pub bucket_id: String,
}

pub async fn setup_test_with_auth() -> TestSetup {
    // Don't want to authorize every time.
    static AUTH: OnceCell<B2Auth> = OnceCell::const_new();

    let client = reqwest::ClientBuilder::new().build().unwrap();
    let auth = AUTH
        .get_or_init(|| async {
            api::b2_authorize_account(&client, std::env::var("B2_TEST_KEY_STRING").unwrap())
                .await
                .unwrap()
        })
        .await
        .clone();
    let bucket_id = std::env::var("B2_TEST_BUCKET_ID").unwrap();
    TestSetup {
        client,
        auth,
        bucket_id,
    }
}

pub async fn open_test_file(name: &str) -> File {
    use std::path::PathBuf;
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/");
    path.push(name);
    let file = tokio::fs::File::open(&path).await.unwrap();
    file
}
