# L{CORE} Event Indexer

A high-performance blockchain event indexer for L{CORE} smart contracts. This service provides real-time indexing of blockchain events from VerifierRegistry, DeviceRegistry, and IoTDataPipeline contracts, offering a fast REST API for querying historical data and analytics.

## Overview

The L{CORE} Event Indexer is a critical infrastructure component that:

- **Real-time Event Indexing**: Monitors blockchain events via WebSocket connections
- **Fast Data Access**: Provides millisecond-response REST API for historical queries
- **Analytics Ready**: Pre-processes data for dashboard and analytics use cases
- **Scalable Storage**: Uses PostgreSQL for efficient querying and data integrity
- **Production Ready**: Includes health monitoring, migrations, and comprehensive logging

## Features

### Core Functionality
- **Real-time Event Indexing**: WebSocket-based blockchain monitoring
- **Multi-Contract Support**: Indexes VerifierRegistry, DeviceRegistry, and IoTDataPipeline
- **REST API**: Comprehensive endpoints for querying indexed data
- **Database Management**: Automatic migrations and connection pooling
- **Health Monitoring**: Built-in health checks and statistics endpoints

### Performance & Reliability
- **PostgreSQL Storage**: Optimized queries with proper indexing
- **Pagination Support**: Efficient handling of large datasets
- **Error Handling**: Robust retry logic and graceful degradation
- **Configurable**: Environment variables and TOML configuration support

## Architecture

```mermaid
flowchart TD
    BC[Blockchain Network] -->|WebSocket Events| EI[Event Indexer]
    EI -->|Store Events| DB[(PostgreSQL Database)]
    EI -->|REST API| API[API Server :8090]
    API -->|JSON Responses| CLIENT[Client Applications]
    
    subgraph "Indexed Events"
        VE[Verifier Events]
        DE[Device Events]  
        DT[Device Transfers]
        DS[Data Submissions]
        MC[Marketplace Config]
        OT[Ownership Transfers]
    end
    
    DB --> VE
    DB --> DE
    DB --> DT
    DB --> DS
    DB --> MC
    DB --> OT
```

## Configuration

### Environment Variables

The indexer supports both prefixed (`INDEXER_*`) and direct environment variables for Railway/cloud deployment compatibility:

```bash
# Database Configuration
DATABASE_URL="postgresql://postgres:password@localhost:5432/lcore_indexer"
INDEXER_DATABASE_URL="postgresql://postgres:password@localhost:5432/lcore_indexer"  # Alternative

# Blockchain Connection
BLOCKCHAIN_WS_URL="ws://localhost:8545"
INDEXER_BLOCKCHAIN_WS_URL="ws://localhost:8545"  # Alternative

# Contract Addresses
VERIFIER_REGISTRY_ADDRESS="0x0000000000000000000000000000000000000000"
DEVICE_REGISTRY_ADDRESS="0x0000000000000000000000000000000000000000"
IOT_PIPELINE_ADDRESS="0x0000000000000000000000000000000000000000"

# Indexing Configuration
START_BLOCK="0"                    # Block to start indexing from
POLL_INTERVAL_SECS="10"           # Polling interval for events
MAX_RETRIES="3"                   # Max retries for network requests
REQUEST_TIMEOUT_SECS="30"         # Request timeout

# API Server
INDEXER_API_HOST="0.0.0.0"        # API server host
INDEXER_API_PORT="8090"           # API server port
```

### Configuration File

Copy `indexer.toml.example` to `indexer.toml` and configure:

```toml
# Database connection
database_url = "postgresql://postgres:password@localhost:5432/lcore_indexer"

# Blockchain WebSocket connection
blockchain_ws_url = "ws://localhost:8545"

# Contract addresses
verifier_registry_address = "0x0000000000000000000000000000000000000000"
device_registry_address = "0x0000000000000000000000000000000000000000"
iot_pipeline_address = "0x0000000000000000000000000000000000000000"

# Starting block for indexing (0 = genesis)
start_block = 0

# API server configuration
api_host = "0.0.0.0"
api_port = 8090
```

## Installation

### Prerequisites

- **Rust 1.82+** with nightly toolchain (for edition2024 support)
- **PostgreSQL 12+** database
- **Access to blockchain RPC** with WebSocket support

### From Source

```bash
# Clone the repository
git clone https://github.com/Modern-Society-Labs/lcore-indexer.git
cd lcore-indexer

# Install Rust nightly (required for edition2024)
rustup toolchain install nightly
rustup default nightly

# Build the project
cargo build --release

# Set up configuration
cp indexer.toml.example indexer.toml
# Edit indexer.toml with your configuration

# Run database migrations
sqlx migrate run --database-url "your_database_url"

# Start the indexer
./target/release/lcore-indexer --config indexer.toml
```

### Using Docker

```bash
# Pull the latest image
docker pull modernsociety/lcore-indexer:latest

# Or build locally
docker build -t lcore-indexer .

# Run with environment variables
docker run -d \
  -e DATABASE_URL="postgresql://postgres:password@localhost:5432/lcore_indexer" \
  -e BLOCKCHAIN_WS_URL="ws://localhost:8545" \
  -e VERIFIER_REGISTRY_ADDRESS="0xYourVerifierRegistryAddress" \
  -e DEVICE_REGISTRY_ADDRESS="0xYourDeviceRegistryAddress" \
  -e IOT_PIPELINE_ADDRESS="0xYourIoTPipelineAddress" \
  -e START_BLOCK="0" \
  -p 8090:8090 \
  --name lcore-indexer \
  modernsociety/lcore-indexer:latest
```

## API Endpoints

The indexer provides a comprehensive REST API for querying indexed blockchain events:

### Health & Monitoring

- **`GET /health`** - Service health check
  ```json
  {
    "status": "healthy",
    "latest_block": 1234567
  }
  ```

- **`GET /stats`** - Indexing statistics and metrics
  ```json
  {
    "verifier_count": 5,
    "device_count": 1250,
    "data_submission_count": 45000,
    "latest_block": 1234567
  }
  ```

### Verifier Management

- **`GET /verifiers`** - List all registered verifiers
- **`GET /verifiers/:address/events`** - Get events for a specific verifier

### Device Registry

- **`GET /devices`** - List all registered devices
- **`GET /devices/:id`** - Get details for a specific device
- **`GET /devices/:id/events`** - Get all events for a device (registration, updates, transfers)
- **`GET /devices/:id/data`** - Get data submissions from a device

### Data Submissions

- **`GET /data/recent`** - Get recent data submissions across all devices

### Governance & Ownership

- **`GET /ownership-transfers`** - Get contract ownership transfer events

## Query Parameters

All list endpoints support pagination and filtering:

### Pagination
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20, max: 100)

### Examples
```bash
# Get second page with 50 items
GET /devices?page=2&limit=50

# Get recent data submissions
GET /data/recent?limit=10

# Get events for a specific device
GET /devices/0x1234.../events?page=1&limit=25
```

## Response Format

All API responses follow a consistent format:

### Successful Responses
```json
{
  "data": [...],           // Array of results
  "page": 1,              // Current page number
  "limit": 20,            // Items per page
  "total": 100            // Total number of items
}
```

### Error Responses
```json
{
  "error": "Bad request",
  "message": "Invalid page parameter"
}
```

### Health Check Response
```json
{
  "status": "healthy",
  "latest_block": 1234567
}
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Database Management

```bash
# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run --database-url "postgresql://postgres:password@localhost:5432/lcore_indexer"

# Revert last migration
sqlx migrate revert --database-url "postgresql://postgres:password@localhost:5432/lcore_indexer"

# Check migration status
sqlx migrate info --database-url "postgresql://postgres:password@localhost:5432/lcore_indexer"
```

### Local Development Setup

```bash
# Start PostgreSQL (using Docker)
docker run -d \
  --name postgres-indexer \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=lcore_indexer \
  -p 5432:5432 \
  postgres:15

# Run migrations
sqlx migrate run --database-url "postgresql://postgres:password@localhost:5432/lcore_indexer"

# Start the indexer in development mode
cargo run -- --config indexer.toml --verbose
```

## Production Deployment

### Prerequisites
1. **PostgreSQL Database**: Set up a production PostgreSQL instance
2. **Blockchain Access**: Ensure WebSocket access to your blockchain network
3. **Contract Addresses**: Deploy and configure L{CORE} smart contracts
4. **Monitoring**: Set up log aggregation and monitoring

### Deployment Steps

#### 1. Database Setup
```bash
# Create production database
createdb lcore_indexer_prod

# Run migrations
sqlx migrate run --database-url "postgresql://user:password@prod-host:5432/lcore_indexer_prod"
```

#### 2. Docker Deployment
```bash
# Pull latest image
docker pull modernsociety/lcore-indexer:latest

# Run in production
docker run -d \
  --name lcore-indexer-prod \
  --restart unless-stopped \
  -e DATABASE_URL="postgresql://user:password@prod-host:5432/lcore_indexer_prod" \
  -e BLOCKCHAIN_WS_URL="wss://your-blockchain-rpc.com/ws" \
  -e VERIFIER_REGISTRY_ADDRESS="0xYourVerifierRegistryAddress" \
  -e DEVICE_REGISTRY_ADDRESS="0xYourDeviceRegistryAddress" \
  -e IOT_PIPELINE_ADDRESS="0xYourIoTPipelineAddress" \
  -e START_BLOCK="1000000" \
  -p 8090:8090 \
  modernsociety/lcore-indexer:latest
```

#### 3. Health Monitoring
```bash
# Check service health
curl http://localhost:8090/health

# Monitor statistics
curl http://localhost:8090/stats
```

### Production Considerations

- **Database Connection Pooling**: Configure appropriate connection limits
- **Error Handling**: Monitor logs for WebSocket disconnections and retry logic
- **Backup Strategy**: Regular database backups for indexed data
- **Load Balancing**: Use multiple instances behind a load balancer for high availability
- **Security**: Use environment variables for sensitive configuration
- **Monitoring**: Set up alerts for service health and indexing lag

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Apache-2.0 - see [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/Modern-Society-Labs/lcore-indexer/issues)
- **Documentation**: [L{CORE} Documentation](https://docs.lcore.io)
- **Community**: [Discord](https://discord.gg/lcore)
