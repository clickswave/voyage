use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Config {
    pub domains: Vec<String>,
    pub interval: i64,
    pub threads: i64,
    pub agent: String,
    pub wordlist_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Scan {
    pub id: String,
    pub config_hash: String,
    pub config: String,
    pub status: String,
    pub no_banner: bool,
    pub launch_delay: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub notifications: String,
}


