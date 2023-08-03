//! # dehashed-rs
//!
//! This crate provides a SDK for the API of [dehashed](https://dehashed.com/).
//!
//! Please note, that this project is not an official implementation from dehashed.
//!
//! ## Usage
//!
//!
//! ```
//! use dehashed_rs::*;
//!
//! let email = "test@example.com".to_string();
//! let api_key = "<api_key>".to_string();
//!
//! // Create an api instance
//! let api = DehashedApi::new(email, api_key).unwrap();
//!
//! // Query for the domain example.com
//! if let Ok(res) = api
//!     .search(Query::Domain(SearchType::Simple("example.com".to_string())))
//!     .await
//! {
//!     println!("{res:?}");
//! }
//! ```
//!
//! or if you enable the `tokio` feature, you can utilize the scheduler to abstract
//! away the need to manage get past the rate limit:
//!
//! ```
//! use dehashed_rs::*;
//! use tokio::sync::oneshot;
//!
//! let email = "test@example.com".to_string();
//! let api_key = "<api_key>".to_string();
//!
//! // Create an api instance
//! let api = DehashedApi::new(email, api_key).unwrap();
//! // Create the scheduler
//! let scheduler = api.start_scheduler();
//!
//! let tx = scheduler.retrieve_sender();
//!
//! let (ret_tx, ret_rx) = oneshot::channel();
//!
//! // Schedule a query for the email "test@example.com"
//! tx.send(ScheduledRequest::new(
//!     Query::Email(SearchType::Simple("test@example.com".to_string())),
//!     ret_tx, //!
//! ))
//! .await
//! .unwrap();
//!
//! // Retrieve the result
//! if let Ok(res) = ret_rx.await {
//!     println!("{res:?}");
//! }
//! ```

#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub use api::*;
pub use error::DehashedError;
#[cfg(feature = "tokio")]
pub use scheduler::*;

mod api;
mod error;
pub(crate) mod res;
#[cfg(feature = "tokio")]
mod scheduler;
#[cfg(test)]
mod tests;
