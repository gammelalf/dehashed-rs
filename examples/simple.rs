use dehashed_rs::{DehashedApi, Query, SearchType};

// Tokio is not needed, but is used as example runtime
#[tokio::main]
async fn main() {
    let email = "test@example.com".to_string();
    let api_key = "<api_key>".to_string();

    // Create an api instance
    let api = DehashedApi::new(email, api_key).unwrap();

    // Query for the domain example.com
    if let Ok(res) = api
        .search(Query::Domain(SearchType::Simple("example.com".to_string())))
        .await
    {
        println!("{res:?}");
    }

    // Query for example.com or example.org
    if let Ok(res) = api
        .search(Query::Domain(SearchType::Or(vec![
            SearchType::Simple("example.com".to_string()),
            SearchType::Simple("example.org".to_string()),
        ])))
        .await
    {
        println!("{res:?}");
    }
}
