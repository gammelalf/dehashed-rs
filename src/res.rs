use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) hashed_password: String,
    pub(crate) ip_address: String,
    pub(crate) name: String,
    pub(crate) vin: String,
    pub(crate) address: String,
    pub(crate) phone: String,
    pub(crate) database_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Response {
    pub(crate) balance: usize,
    pub(crate) entries: Vec<Entry>,
    pub(crate) success: bool,
    pub(crate) took: String,
    pub(crate) total: usize,
}
