use std::fmt::Write;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use log::{debug, error};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
#[cfg(feature = "tokio")]
use tokio::time::sleep;

use crate::error::DehashedError;
use crate::res::{Entry, Response};
#[cfg(feature = "tokio")]
use crate::Scheduler;

const URL: &str = "https://api.dehashed.com/search";
const RESERVED: [char; 21] = [
    '+', '-', '=', '&', '|', '>', '<', '!', '(', ')', '{', '}', '[', ']', '^', '"', '~', '*', '?',
    ':', '\\',
];

fn escape(q: &str) -> String {
    let mut s = String::new();
    for c in q.chars() {
        if RESERVED.contains(&c) {
            s.write_str(&format!("\\{c}")).unwrap();
        } else {
            s.write_char(c).unwrap();
        }
    }
    s
}

/// A specific search type
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum SearchType {
    /// Search for a simple pattern
    Simple(String),
    /// Search for an exact pattern
    Exact(String),
    /// A regex search pattern
    Regex(String),
    /// Add multiple [SearchType]s with an OR
    Or(Vec<SearchType>),
    /// Add multiple [SearchType]s with an AND
    And(Vec<SearchType>),
}

impl ToString for SearchType {
    fn to_string(&self) -> String {
        match self {
            SearchType::Simple(x) => escape(x),
            SearchType::Exact(x) => format!("\"{}\"", escape(x)),
            SearchType::Regex(x) => format!("/{}/", escape(x)),
            SearchType::Or(x) => x
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" OR "),
            SearchType::And(x) => x
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}

/// A query for dehashed
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum Query {
    /// Search for an email
    Email(SearchType),
    /// Search for an ip address
    IpAddress(SearchType),
    /// Search for an username
    Username(SearchType),
    /// Search for an password
    Password(SearchType),
    /// Search for an hashed password
    HashedPassword(SearchType),
    /// Search for a name
    Name(SearchType),
    /// Search for a domain
    Domain(SearchType),
    /// Search for a vin
    Vin(SearchType),
    /// Search for a phone
    Phone(SearchType),
    /// Search for an address
    Address(SearchType),
}

impl ToString for Query {
    fn to_string(&self) -> String {
        match self {
            Query::Email(x) => format!("email:{}", x.to_string()),
            Query::IpAddress(x) => format!("ip_address:{}", x.to_string()),
            Query::Username(x) => format!("username:{}", x.to_string()),
            Query::Password(x) => format!("password:{}", x.to_string()),
            Query::HashedPassword(x) => format!("hashed_password:{}", x.to_string()),
            Query::Name(x) => format!("name:{}", x.to_string()),
            Query::Domain(x) => format!("domain:{}", x.to_string()),
            Query::Vin(x) => format!("vin:{}", x.to_string()),
            Query::Phone(x) => format!("phone:{}", x.to_string()),
            Query::Address(x) => format!("address:{}", x.to_string()),
        }
    }
}

/// The result of a search query
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SearchResult {
    /// A list of results
    pub entries: Vec<SearchEntry>,
    /// The remaining balance
    pub balance: usize,
}

/// A single entry in a [SearchResult]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct SearchEntry {
    /// ID of the entry
    pub id: u64,
    /// An email address, may be [None] if the result didn't include this field
    pub email: Option<String>,
    /// An username, may be [None] if the result didn't include this field
    pub username: Option<String>,
    /// A password, may be [None] if the result didn't include this field
    pub password: Option<String>,
    /// An hashed password, may be [None] if the result didn't include this field
    pub hashed_password: Option<String>,
    /// An ip address, may be [None] if the result didn't include this field
    pub ip_address: Option<IpAddr>,
    /// A name, may be [None] if the result didn't include this field
    pub name: Option<String>,
    /// A vin, may be [None] if the result didn't include this field
    pub vin: Option<String>,
    /// An address, may be [None] if the result didn't include this field
    pub address: Option<String>,
    /// A phone, may be [None] if the result didn't include this field
    pub phone: Option<String>,
    /// A database name, may be [None] if the result didn't include this field
    pub database_name: Option<String>,
}

impl TryFrom<Entry> for SearchEntry {
    type Error = DehashedError;

    fn try_from(value: Entry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.parse()?,
            email: if value.email.is_empty() {
                None
            } else {
                Some(value.email)
            },
            username: if value.username.is_empty() {
                None
            } else {
                Some(value.username)
            },
            password: if value.password.is_empty() {
                None
            } else {
                Some(value.password)
            },
            hashed_password: if value.hashed_password.is_empty() {
                None
            } else {
                Some(value.hashed_password)
            },
            ip_address: if value.ip_address.is_empty() {
                None
            } else {
                Some(IpAddr::from_str(&value.ip_address)?)
            },
            name: if value.name.is_empty() {
                None
            } else {
                Some(value.name)
            },
            vin: if value.vin.is_empty() {
                None
            } else {
                Some(value.vin)
            },
            address: if value.address.is_empty() {
                None
            } else {
                Some(value.address)
            },
            phone: if value.phone.is_empty() {
                None
            } else {
                Some(value.phone)
            },
            database_name: if value.database_name.is_empty() {
                None
            } else {
                Some(value.database_name)
            },
        })
    }
}

/// The instance of the dehashed api
#[derive(Clone, Debug)]
pub struct DehashedApi {
    email: String,
    api_key: String,
    client: Client,
}

impl DehashedApi {
    /// Create a new instance of the SDK.
    ///
    /// **Parameter**:
    /// - `email`: The mail address that is used for authentication
    /// - `api_key`: The api key for your account (found on your profile page)
    ///
    /// This method fails if the [Client] could not be constructed
    pub fn new(email: String, api_key: String) -> Result<Self, DehashedError> {
        let mut header_map = HeaderMap::new();
        header_map.insert("Accept", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .https_only(true)
            .default_headers(header_map)
            .build()?;

        Ok(Self {
            email,
            client,
            api_key: api_key.to_lowercase(),
        })
    }

    async fn raw_req(
        &self,
        size: usize,
        page: usize,
        query: String,
    ) -> Result<Response, DehashedError> {
        let res = self
            .client
            .get(URL)
            .basic_auth(&self.email, Some(&self.api_key))
            .query(&[
                ("size", size.to_string()),
                ("query", query),
                ("page", page.to_string()),
            ])
            .send()
            .await?;

        let status = res.status();
        if status == StatusCode::from_u16(302).unwrap() {
            Err(DehashedError::InvalidQuery)
        } else if status == StatusCode::from_u16(400).unwrap() {
            Err(DehashedError::RateLimited)
        } else if status == StatusCode::from_u16(401).unwrap() {
            Err(DehashedError::Unauthorized)
        } else if status == StatusCode::from_u16(200).unwrap() {
            let raw = res.text().await?;

            match serde_json::from_str(&raw) {
                Ok(result) => Ok(result),
                Err(err) => {
                    error!("Error deserializing data: {err}. Raw data: {raw}");
                    Err(DehashedError::Unknown)
                }
            }
        } else {
            Err(DehashedError::Unknown)
        }
    }

    /// Query the API
    ///
    /// Please note, that dehashed has a ratelimit protection active, that bans every account
    /// that is doing more than 5 req / s.
    ///
    /// This method will take care of pagination and will delay requests if necessary.
    pub async fn search(&self, query: Query) -> Result<SearchResult, DehashedError> {
        let q = query.to_string();
        debug!("Query: {q}");

        let mut search_result = SearchResult {
            entries: vec![],
            balance: 0,
        };
        for page in 1.. {
            let res = self.raw_req(10_000, page, q.clone()).await?;

            if !res.success {
                error!("Success field in response is set to false");
                return Err(DehashedError::Unknown);
            }

            if let Some(entries) = res.entries {
                for entry in entries {
                    search_result.entries.push(entry.try_into()?)
                }
            }

            search_result.balance = res.balance;

            if res.total < page * 10_000 {
                break;
            }

            #[cfg(feature = "tokio")]
            sleep(Duration::from_millis(200)).await;
        }

        Ok(search_result)
    }

    /// Start a new scheduler.
    ///
    /// The [Scheduler] manages stay in bounds of the rate limit of the unhashed API.
    /// It lets you push queries and receive the results.
    #[cfg(feature = "tokio")]
    pub fn start_scheduler(&self) -> Scheduler {
        Scheduler::new(self)
    }
}
