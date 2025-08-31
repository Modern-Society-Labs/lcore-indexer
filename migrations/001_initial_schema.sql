-- Initial schema for L{CORE} Event Indexer

-- Verifier events
CREATE TABLE IF NOT EXISTS verifier_events (
    id BIGSERIAL PRIMARY KEY,
    verifier_address VARCHAR(42) NOT NULL,
    event_type VARCHAR(20) NOT NULL CHECK (event_type IN ('added', 'removed')),
    timestamp BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_verifier_events_address ON verifier_events(verifier_address);
CREATE INDEX idx_verifier_events_timestamp ON verifier_events(timestamp);
CREATE INDEX idx_verifier_events_block ON verifier_events(block_number);

-- Device events
CREATE TABLE IF NOT EXISTS device_events (
    id BIGSERIAL PRIMARY KEY,
    device_id VARCHAR(64) NOT NULL,
    owner_address VARCHAR(42) NOT NULL,
    event_type VARCHAR(20) NOT NULL CHECK (event_type IN ('registered', 'updated', 'transferred')),
    device_type INTEGER,
    zone VARCHAR(100),
    timestamp BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_device_events_device_id ON device_events(device_id);
CREATE INDEX idx_device_events_owner ON device_events(owner_address);
CREATE INDEX idx_device_events_timestamp ON device_events(timestamp);
CREATE INDEX idx_device_events_block ON device_events(block_number);

-- Device transfers
CREATE TABLE IF NOT EXISTS device_transfers (
    id BIGSERIAL PRIMARY KEY,
    device_id VARCHAR(64) NOT NULL,
    old_owner VARCHAR(42) NOT NULL,
    new_owner VARCHAR(42) NOT NULL,
    timestamp BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_device_transfers_device_id ON device_transfers(device_id);
CREATE INDEX idx_device_transfers_old_owner ON device_transfers(old_owner);
CREATE INDEX idx_device_transfers_new_owner ON device_transfers(new_owner);

-- Data submissions
CREATE TABLE IF NOT EXISTS data_submissions (
    id BIGSERIAL PRIMARY KEY,
    data_hash VARCHAR(64) NOT NULL,
    device_id_hash VARCHAR(64) NOT NULL,
    device_owner VARCHAR(42) NOT NULL,
    timestamp BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_data_submissions_data_hash ON data_submissions(data_hash);
CREATE INDEX idx_data_submissions_device_hash ON data_submissions(device_id_hash);
CREATE INDEX idx_data_submissions_owner ON data_submissions(device_owner);
CREATE INDEX idx_data_submissions_timestamp ON data_submissions(timestamp);

-- Marketplace config updates
CREATE TABLE IF NOT EXISTS marketplace_config (
    id BIGSERIAL PRIMARY KEY,
    base_fee BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Ownership transfers
CREATE TABLE IF NOT EXISTS ownership_transfers (
    id BIGSERIAL PRIMARY KEY,
    contract_type VARCHAR(50) NOT NULL,
    previous_owner VARCHAR(42) NOT NULL,
    new_owner VARCHAR(42) NOT NULL,
    block_number BIGINT NOT NULL,
    tx_hash VARCHAR(66) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_ownership_transfers_contract ON ownership_transfers(contract_type);
CREATE INDEX idx_ownership_transfers_prev_owner ON ownership_transfers(previous_owner);
CREATE INDEX idx_ownership_transfers_new_owner ON ownership_transfers(new_owner);
