use sqlx::{FromRow};
use chrono::{NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Result {
    pub id: i32,
    pub subdomain: String,
    pub domain: String,
    pub status: Option<String>,
    pub created_on: NaiveDateTime,
    pub last_scanned_on: Option<NaiveDateTime>,
    pub last_scan_started_on: Option<NaiveDateTime>,
    pub max_retries: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SlimResult {
    pub id: i32,
    pub subdomain: String,
    pub domain: String,
    pub max_retries: i32,
}

