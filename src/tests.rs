use std::env;

use crate::api::{Query, SearchType};
use crate::DehashedApi;

#[tokio::test]
async fn test() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    env_logger::init();

    let email = env::var("EMAIL").unwrap();
    let api_key = env::var("API_KEY").unwrap();
    let search = env::var("SEARCH").unwrap();

    let api = DehashedApi::new(email, api_key).unwrap();

    api.search(Query::Domain(SearchType::Simple(search)))
        .await
        .unwrap();
}
