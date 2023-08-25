# dehashed-rs

[![LICENSE](https://img.shields.io/github/license/myOmikron/dehashed-rs?color=blue)](LICENSE)
[![dependency status](https://deps.rs/repo/github/myOmikron/dehashed-rs/status.svg)](https://deps.rs/repo/github/myOmikron/dehashed-rs)
[![ci status](https://img.shields.io/github/actions/workflow/status/myOmikron/dehashed-rs/linux.yml?label=CI)](https://github.com/myOmikron/dehashed-rs/actions/workflows/linux.yml)
[![Docs](https://img.shields.io/docsrs/dehashed-rs?label=Docs)](https://docs.rs/dehashed-rs/latest/)

This is an SDK for the [dehashed](https://dehashed.com/) api.

## Usage

```rs
use dehashed_rs::*;

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
```

or if you enable the `tokio` feature, you can utilize the scheduler to abstract
away the need to manage get past the rate limit:

```rs
use dehashed_rs::*;
use tokio::sync::oneshot;

let email = "test@example.com".to_string();
let api_key = "<api_key>".to_string();

// Create an api instance
let api = DehashedApi::new(email, api_key).unwrap();
// Create the scheduler
let scheduler = api.start_scheduler();

let tx = scheduler.retrieve_sender();

let (ret_tx, ret_rx) = oneshot::channel();

// Schedule a query for the email "test@example.com"
tx.send(ScheduledRequest::new(
    Query::Email(SearchType::Simple("test@example.com".to_string())),
    ret_tx, 
))
.await
.unwrap();

// Retrieve the result
if let Ok(res) = ret_rx.await {
    println!("{res:?}");
}
```

If you need type definitions for utoipa, there available under the feature flag `utoipa`.

## Note

**This is not an official API**
