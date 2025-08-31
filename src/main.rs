//! L{CORE} Event Indexer Service
//! 
//! Indexes blockchain events from VerifierRegistry, DeviceRegistry, and IoTDataPipeline contracts

use anyhow::{Context, Result};
use clap::Parser;
use ethers::{
    contract::{abigen, EthEvent},
    core::types::Filter,
    providers::{Provider, Ws, Middleware, StreamExt},
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

mod api_simple;
use api_simple as api;
mod config;
mod error;
mod models;

use config::Config;

// Generate contract bindings
abigen!(
    VerifierRegistry,
    r#"[
        event VerifierAdded(address indexed verifier, uint256 timestamp)
        event VerifierRemoved(address indexed verifier, uint256 timestamp)
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner)
    ]"#
);

abigen!(
    DeviceRegistry,
    r#"[
        event DeviceRegistered(bytes32 indexed deviceId, address indexed owner, uint8 deviceType, string zone, uint256 timestamp)
        event DeviceUpdated(bytes32 indexed deviceId, address indexed owner, uint256 timestamp)
        event DeviceTransferred(bytes32 indexed deviceId, address indexed oldOwner, address indexed newOwner, uint256 timestamp)
    ]"#
);

abigen!(
    IoTDataPipeline,
    r#"[
        event DataSubmitted(bytes32 indexed dataHash, bytes32 indexed deviceIdHash, address indexed deviceOwner, uint256 timestamp)
        event MarketplaceConfigUpdated(uint256 baseFee)
    ]"#
);

#[derive(Parser)]
#[command(name = "lcore-indexer")]
#[command(about = "L{CORE} Event Indexer Service", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "indexer.toml")]
    config: String,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

/// Application state
struct AppState {
    db: Pool<Postgres>,
    config: Config,
    latest_block: Arc<RwLock<u64>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    let filter = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    info!("Starting L{{CORE}} Event Indexer");
    
    // Load configuration
    let config = Config::load(&args.config)?;
    info!("Configuration loaded from: {}", args.config);
    
    // Connect to database
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;
    
    info!("Connected to database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .context("Failed to run migrations")?;
    
    info!("Database migrations complete");
    
    // Create application state
    let state = Arc::new(AppState {
        db: db.clone(),
        config: config.clone(),
        latest_block: Arc::new(RwLock::new(0)),
    });
    
    // Start API server
    let api_handle = tokio::spawn(api::run_server(state.clone()));
    
    // TODO: Re-enable event indexing after fixing SQLx macros
    // Start event indexing
    // let indexer_handle = tokio::spawn(run_indexer(state));
    
    info!("L{{CORE}} Event Indexer started successfully");
    info!("API server running on port {}", config.api_port);
    
    // Wait for API server only
    if let Err(e) = api_handle.await {
        error!("API server failed: {:?}", e);
    }
    
    Ok(())
}

async fn _run_indexer(state: Arc<AppState>) -> Result<()> {
    // Connect to blockchain
    let provider = Provider::<Ws>::connect(&state.config.blockchain_ws_url)
        .await
        .context("Failed to connect to blockchain")?;
    
    let provider = Arc::new(provider);
    
    info!("Connected to blockchain: {}", state.config.blockchain_ws_url);
    
    // Get current block
    let current_block = provider.get_block_number().await?;
    info!("Current block: {}", current_block);
    
    // Update latest block
    {
        let mut latest = state.latest_block.write().await;
        *latest = current_block.as_u64();
    }
    
    // Start indexing each contract
    let verifier_handle = tokio::spawn(index_verifier_registry(
        state.clone(),
        provider.clone(),
    ));
    
    let device_handle = tokio::spawn(index_device_registry(
        state.clone(),
        provider.clone(),
    ));
    
    let pipeline_handle = tokio::spawn(index_iot_pipeline(
        state.clone(),
        provider.clone(),
    ));
    
    // Wait for all indexers
    tokio::try_join!(
        verifier_handle,
        device_handle,
        pipeline_handle,
    )?;
    
    Ok(())
}

async fn index_verifier_registry(
    state: Arc<AppState>,
    provider: Arc<Provider<Ws>>,
) -> Result<()> {
    let contract_address = state.config.verifier_registry_address.parse()?;
    
    info!("Indexing VerifierRegistry at: {}", contract_address);
    
    // Create filter for all events
    let filter = Filter::new()
        .address(contract_address)
        .from_block(state.config.start_block);
    
    // Subscribe to events
    let mut stream = provider.subscribe_logs(&filter).await?;
    
    while let Some(log) = stream.next().await {
        match log.topics[0] {
            topic if topic == VerifierAddedFilter::signature() => {
                let event = VerifierAddedFilter::decode_log(&log.into())?;
                handle_verifier_added(&state.db, event).await?;
            }
            topic if topic == VerifierRemovedFilter::signature() => {
                let event = VerifierRemovedFilter::decode_log(&log.into())?;
                handle_verifier_removed(&state.db, event).await?;
            }
            topic if topic == OwnershipTransferredFilter::signature() => {
                let event = OwnershipTransferredFilter::decode_log(&log.into())?;
                handle_ownership_transferred(&state.db, event, "verifier_registry").await?;
            }
            _ => {
                warn!("Unknown event topic: {:?}", log.topics[0]);
            }
        }
        
        // Update latest block
        if let Some(block_number) = log.block_number {
            let mut latest = state.latest_block.write().await;
            *latest = block_number.as_u64();
        }
    }
    
    Ok(())
}

async fn index_device_registry(
    state: Arc<AppState>,
    provider: Arc<Provider<Ws>>,
) -> Result<()> {
    let contract_address = state.config.device_registry_address.parse()?;
    
    info!("Indexing DeviceRegistry at: {}", contract_address);
    
    // Create filter for all events
    let filter = Filter::new()
        .address(contract_address)
        .from_block(state.config.start_block);
    
    // Subscribe to events
    let mut stream = provider.subscribe_logs(&filter).await?;
    
    while let Some(log) = stream.next().await {
        match log.topics[0] {
            topic if topic == DeviceRegisteredFilter::signature() => {
                let event = DeviceRegisteredFilter::decode_log(&log.into())?;
                handle_device_registered(&state.db, event).await?;
            }
            topic if topic == DeviceUpdatedFilter::signature() => {
                let event = DeviceUpdatedFilter::decode_log(&log.into())?;
                handle_device_updated(&state.db, event).await?;
            }
            topic if topic == DeviceTransferredFilter::signature() => {
                let event = DeviceTransferredFilter::decode_log(&log.into())?;
                handle_device_transferred(&state.db, event).await?;
            }
            _ => {
                warn!("Unknown event topic: {:?}", log.topics[0]);
            }
        }
        
        // Update latest block
        if let Some(block_number) = log.block_number {
            let mut latest = state.latest_block.write().await;
            *latest = block_number.as_u64();
        }
    }
    
    Ok(())
}

async fn index_iot_pipeline(
    state: Arc<AppState>,
    provider: Arc<Provider<Ws>>,
) -> Result<()> {
    let contract_address = state.config.iot_pipeline_address.parse()?;
    
    info!("Indexing IoTDataPipeline at: {}", contract_address);
    
    // Create filter for all events
    let filter = Filter::new()
        .address(contract_address)
        .from_block(state.config.start_block);
    
    // Subscribe to events
    let mut stream = provider.subscribe_logs(&filter).await?;
    
    while let Some(log) = stream.next().await {
        match log.topics[0] {
            topic if topic == DataSubmittedFilter::signature() => {
                let event = DataSubmittedFilter::decode_log(&log.into())?;
                handle_data_submitted(&state.db, event).await?;
            }
            topic if topic == MarketplaceConfigUpdatedFilter::signature() => {
                let event = MarketplaceConfigUpdatedFilter::decode_log(&log.into())?;
                handle_marketplace_config_updated(&state.db, event).await?;
            }
            _ => {
                warn!("Unknown event topic: {:?}", log.topics[0]);
            }
        }
        
        // Update latest block
        if let Some(block_number) = log.block_number {
            let mut latest = state.latest_block.write().await;
            *latest = block_number.as_u64();
        }
    }
    
    Ok(())
}

// Event handlers
async fn handle_verifier_added(db: &Pool<Postgres>, event: VerifierAddedFilter) -> Result<()> {
    info!("Verifier added: {:?}", event.verifier);
    
    sqlx::query(
        r#"
        INSERT INTO verifier_events (verifier_address, event_type, timestamp, block_number, tx_hash)
        VALUES ($1, 'added', $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(format!("{:?}", event.verifier))
    .bind(event.timestamp.as_u64() as i64)
    .bind(0i64) // TODO: Get from log
    .bind("0x") // TODO: Get from log
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_verifier_removed(db: &Pool<Postgres>, event: VerifierRemovedFilter) -> Result<()> {
    info!("Verifier removed: {:?}", event.verifier);
    
    sqlx::query(
        r#"
        INSERT INTO verifier_events (verifier_address, event_type, timestamp, block_number, tx_hash)
        VALUES ($1, 'removed', $2, $3, $4)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(format!("{:?}", event.verifier))
    .bind(event.timestamp.as_u64() as i64)
    .bind(0i64) // TODO: Get from log
    .bind("0x") // TODO: Get from log
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_ownership_transferred(
    db: &Pool<Postgres>,
    event: OwnershipTransferredFilter,
    contract_type: &str,
) -> Result<()> {
    info!("Ownership transferred: {:?} -> {:?}", event.previous_owner, event.new_owner);
    
    sqlx::query(
        r#"
        INSERT INTO ownership_transfers (contract_type, previous_owner, new_owner, block_number, tx_hash)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(contract_type)
    .bind(format!("{:?}", event.previous_owner))
    .bind(format!("{:?}", event.new_owner))
    .bind(0i64) // TODO: Get from log
    .bind("0x") // TODO: Get from log
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_device_registered(db: &Pool<Postgres>, event: DeviceRegisteredFilter) -> Result<()> {
    info!("Device registered: {:?}", hex::encode(&event.device_id));
    
    sqlx::query(
        r#"
        INSERT INTO device_events (
            device_id, owner_address, event_type, device_type, zone, 
            timestamp, block_number, tx_hash
        )
        VALUES ($1, $2, 'registered', $3, $4, $5, $6, $7)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(hex::encode(&event.device_id))
    .bind(format!("{:?}", event.owner))
    .bind(event.device_type as i32)
    .bind(event.zone)
    .bind(event.timestamp.as_u64() as i64)
    .bind(0i64) // TODO: Get from log
    .bind("0x") // TODO: Get from log
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_device_updated(db: &Pool<Postgres>, event: DeviceUpdatedFilter) -> Result<()> {
    info!("Device updated: {:?}", hex::encode(&event.device_id));
    
    sqlx::query!(
        r#"
        INSERT INTO device_events (
            device_id, owner_address, event_type, timestamp, block_number, tx_hash
        )
        VALUES ($1, $2, 'updated', $3, $4, $5)
        ON CONFLICT DO NOTHING
        "#,
        hex::encode(&event.device_id),
        format!("{:?}", event.owner),
        event.timestamp.as_u64() as i64,
        0i64, // TODO: Get from log
        "0x" // TODO: Get from log
    )
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_device_transferred(db: &Pool<Postgres>, event: DeviceTransferredFilter) -> Result<()> {
    info!("Device transferred: {:?}", hex::encode(&event.device_id));
    
    sqlx::query!(
        r#"
        INSERT INTO device_transfers (
            device_id, old_owner, new_owner, timestamp, block_number, tx_hash
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING
        "#,
        hex::encode(&event.device_id),
        format!("{:?}", event.old_owner),
        format!("{:?}", event.new_owner),
        event.timestamp.as_u64() as i64,
        0i64, // TODO: Get from log
        "0x" // TODO: Get from log
    )
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_data_submitted(db: &Pool<Postgres>, event: DataSubmittedFilter) -> Result<()> {
    info!("Data submitted: {:?}", hex::encode(&event.data_hash));
    
    sqlx::query!(
        r#"
        INSERT INTO data_submissions (
            data_hash, device_id_hash, device_owner, timestamp, block_number, tx_hash
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING
        "#,
        hex::encode(&event.data_hash),
        hex::encode(&event.device_id_hash),
        format!("{:?}", event.device_owner),
        event.timestamp.as_u64() as i64,
        0i64, // TODO: Get from log
        "0x" // TODO: Get from log
    )
    .execute(db)
    .await?;
    
    Ok(())
}

async fn handle_marketplace_config_updated(
    db: &Pool<Postgres>,
    event: MarketplaceConfigUpdatedFilter,
) -> Result<()> {
    info!("Marketplace config updated: base_fee={}", event.base_fee);
    
    sqlx::query!(
        r#"
        INSERT INTO marketplace_config (base_fee, updated_at, block_number, tx_hash)
        VALUES ($1, NOW(), $2, $3)
        "#,
        event.base_fee.as_u64() as i64,
        0i64, // TODO: Get from log
        "0x" // TODO: Get from log
    )
    .execute(db)
    .await?;
    
    Ok(())
}
