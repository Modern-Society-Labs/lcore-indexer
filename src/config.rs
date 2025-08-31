//! Configuration module for the event indexer

use anyhow::Result;
use config::{Config as ConfigBuilder, ConfigError, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database connection URL
    #[serde(alias = "DATABASE_URL")]
    pub database_url: String,
    
    /// Blockchain WebSocket URL
    #[serde(alias = "BLOCKCHAIN_WS_URL")]
    pub blockchain_ws_url: String,
    
    /// Contract addresses
    #[serde(alias = "VERIFIER_REGISTRY_ADDRESS")]
    pub verifier_registry_address: String,
    #[serde(alias = "DEVICE_REGISTRY_ADDRESS")]
    pub device_registry_address: String,
    #[serde(alias = "IOT_PIPELINE_ADDRESS")]
    pub iot_pipeline_address: String,
    
    /// Starting block for indexing
    #[serde(alias = "START_BLOCK")]
    pub start_block: u64,
    
    /// API server configuration
    #[serde(alias = "INDEXER_API_HOST")]
    pub api_host: String,
    #[serde(alias = "INDEXER_API_PORT")]
    pub api_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:password@localhost/lcore_indexer".to_string(),
            blockchain_ws_url: "ws://localhost:8545".to_string(),
            verifier_registry_address: "0x0000000000000000000000000000000000000000".to_string(),
            device_registry_address: "0x0000000000000000000000000000000000000000".to_string(),
            iot_pipeline_address: "0x0000000000000000000000000000000000000000".to_string(),
            start_block: 0,
            api_host: "0.0.0.0".to_string(),
            api_port: 8090,
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config = ConfigBuilder::builder()
            .set_default("api_host", "0.0.0.0")?
            .set_default("api_port", 8090)?
            .set_default("start_block", 0)?
            .add_source(File::with_name(path).required(false))
            // Add environment variables without prefix first (for Railway compatibility)
            .add_source(config::Environment::default())
            // Then add prefixed environment variables (for local development)
            .add_source(config::Environment::with_prefix("INDEXER"))
            .build()?;
        
        config.try_deserialize().map_err(|e| e.into())
    }
    
    /// Load configuration from environment variables only (for Railway)
    pub fn from_env() -> Result<Self> {
        let config = ConfigBuilder::builder()
            .set_default("api_host", "0.0.0.0")?
            .set_default("api_port", 8090)?
            .set_default("start_block", 0)?
            // Read directly from environment variables
            .add_source(config::Environment::default())
            .build()?;
        
        config.try_deserialize().map_err(|e| e.into())
    }
}
