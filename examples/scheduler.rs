use dehashed_rs::{DehashedApi, Query, ScheduledRequest, SearchType};
use tokio::sync::oneshot;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let email = "test@example.com".to_string();
    let api_key = "<api_key>".to_string();

    let mut set = JoinSet::new();

    // Create an api instance
    let api = DehashedApi::new(email, api_key).unwrap();
    // Create the scheduler
    let scheduler = api.start_scheduler();

    // Clone the scheduler
    let s = scheduler.clone();
    set.spawn(async move {
        let tx = s.retrieve_sender();
        let (ret_tx, ret_rx) = oneshot::channel();

        // Schedule a search for the domain example.com or example.org
        tx.send(ScheduledRequest::new(
            Query::Domain(SearchType::Or(vec![
                SearchType::Simple("example.com".to_string()),
                SearchType::Exact("example.org".to_string()),
            ])),
            ret_tx,
        ))
        .await
        .unwrap();

        if let Ok(result) = ret_rx.await {
            println!("{result:?}");
        }
    });

    let sender = scheduler.retrieve_sender();

    // or just clone the sender
    let tx = sender.clone();
    set.spawn(async move {
        let (ret_tx, ret_rx) = oneshot::channel();

        tx.send(ScheduledRequest::new(
            Query::Email(SearchType::Simple("test@example.com".to_string())),
            ret_tx,
        ))
        .await
        .unwrap();

        if let Ok(res) = ret_rx.await {
            println!("{res:?}");
        }
    });

    // Wait for all tasks
    while let Some(Ok(_)) = set.join_next().await {}
}
