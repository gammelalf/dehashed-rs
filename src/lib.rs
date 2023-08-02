//! # dehashed-rs
//!
//! This crate provides a SDK for the API of [dehashed](https://dehashed.com/).
//!
//! Please note, that this project is not an official implementation from dehashed.
//!
//! ## Usage

#![warn(missing_docs)]

pub use api::DehashedApi;
pub use error::DehashedError;

mod api;
mod error;
pub(crate) mod res;
#[cfg(test)]
mod tests;
