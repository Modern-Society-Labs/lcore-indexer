//! Data models for the event indexer

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub latest_block: u64,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub verifier_count: i64,
    pub device_count: i64,
    pub data_submission_count: i64,
    pub latest_block: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifierInfo {
    pub address: String,
    pub registered_at: i64,
    pub removed_at: Option<i64>,
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum VerifierEventType {
    Added,
    Removed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifierEvent {
    pub id: i64,
    pub verifier_address: String,
    pub event_type: VerifierEventType,
    pub timestamp: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub owner_address: String,
    pub registered_at: i64,
    pub device_type: Option<i32>,
    pub zone: Option<String>,
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
pub enum DeviceEventType {
    Registered,
    Updated,
    Transferred,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub id: i64,
    pub device_id: String,
    pub owner_address: String,
    pub event_type: DeviceEventType,
    pub device_type: Option<i32>,
    pub zone: Option<String>,
    pub timestamp: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceTransfer {
    pub id: i64,
    pub device_id: String,
    pub old_owner: String,
    pub new_owner: String,
    pub timestamp: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSubmission {
    pub id: i64,
    pub data_hash: String,
    pub device_id_hash: String,
    pub device_owner: String,
    pub timestamp: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub id: i64,
    pub base_fee: i64,
    pub block_number: i64,
    pub tx_hash: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnershipTransfer {
    pub id: i64,
    pub contract_type: String,
    pub previous_owner: String,
    pub new_owner: String,
    pub block_number: i64,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}
