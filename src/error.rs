use std::fmt::{Display, Formatter};
use std::net::AddrParseError;

/// The common error type of this crate
#[derive(Debug)]
pub enum DehashedError {
    /// Error that are caused by reqwest
    ReqwestError(reqwest::Error),
    /// Invalid API credentials
    Unauthorized,
    /// Query is missing or invalid
    InvalidQuery,
    /// The used account got rate limited
    RateLimited,
    /// An unknown error occurred
    Unknown,
    /// An error occurred while parsing an int field
    ParseIntError(std::num::ParseIntError),
    /// An error occurred while parsing an ip addr field
    ParseAddrError(AddrParseError),
}

impl Display for DehashedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DehashedError::ReqwestError(err) => write!(f, "Reqwest error occurred: {err}"),
            DehashedError::Unauthorized => write!(f, "Invalid API credentials"),
            DehashedError::InvalidQuery => write!(f, "The provided query is missing or invalid"),
            DehashedError::RateLimited => write!(f, "The account got rate limited"),
            DehashedError::Unknown => write!(f, "An unknown error occurred"),
            DehashedError::ParseIntError(err) => {
                write!(f, "An error occurred while parsing a response: {err}")
            }
            DehashedError::ParseAddrError(err) => write!(f, "Error while parsing ip addr: {err}"),
        }
    }
}

impl std::error::Error for DehashedError {}

impl From<reqwest::Error> for DehashedError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<std::num::ParseIntError> for DehashedError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<AddrParseError> for DehashedError {
    fn from(value: AddrParseError) -> Self {
        Self::ParseAddrError(value)
    }
}
