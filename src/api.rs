use std::fmt::Write;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};

use crate::error::DehashedError;
use crate::res::{Entry, Response};

const URL: &str = "https://api.dehashed.com/search";
const RESERVED: [char; 21] = [
    '+', '-', '=', '&', '|', '>', '<', '!', '(', ')', '{', '}', '[', ']', '^', '"', '~', '*', '?',
    ':', '\\',
];

fn escape(q: &str) -> String {
    let mut s = String::new();
    for c in q.chars() {
        if RESERVED.contains(&c) {
            s.write_str("\\{c}").unwrap();
        }
    }
    s
}

#[derive(Clone, Debug)]
pub enum SearchType {
    Simple(String),
    Exact(String),
    Regex(String),
    Or(Vec<SearchType>),
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

#[derive(Clone, Debug)]
pub enum Query {
    Email(SearchType),
    IpAddress(SearchType),
    Username(SearchType),
    Password(SearchType),
    HashedPassword(SearchType),
    Name(SearchType),
    Domain(SearchType),
    Vin(SearchType),
    Phone(SearchType),
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
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// A list of results
    pub entries: Vec<SearchEntry>,
    /// The remaining balance
    pub balance: usize,
}

/// A single entry in a [SearchResult]
#[derive(Debug, Clone)]
pub struct SearchEntry {
    /// ID of the entry
    pub id: u64,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub hashed_password: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub name: Option<String>,
    pub vin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
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

/// The instance
/// TODO:
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
            Ok(res.json().await?)
        } else {
            Err(DehashedError::Unknown)
        }
    }

    /// Query the API
    ///
    /// Please note, that dehashed has a ratelimit protection active, that bans every account
    /// that is doing more than 5 req / s.
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

            for entry in res.entries {
                search_result.entries.push(entry.try_into()?)
            }

            search_result.balance = res.balance;

            if res.total < page * 10_000 {
                break;
            }
        }

        Ok(search_result)
    }
}
