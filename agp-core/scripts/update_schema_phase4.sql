-- AGP-CORE Phase 4: Decentralized Architecture & Token Economics
-- Blockchain integration schema

-- 1. Blockchain Networks
CREATE TABLE IF NOT EXISTS blockchain_networks (
    network_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    network_name VARCHAR(100) NOT NULL,
    chain_id INTEGER NOT NULL,
    network_type VARCHAR(50) NOT NULL,  -- 'mainnet', 'testnet', 'sidechain', 'layer2'
    rpc_endpoint VARCHAR(500) NOT NULL,
    wss_endpoint VARCHAR(500),
    explorer_url VARCHAR(500),
    
    -- Token standards
    supports_erc20 BOOLEAN DEFAULT TRUE,
    supports_erc721 BOOLEAN DEFAULT FALSE,
    supports_erc1155 BOOLEAN DEFAULT FALSE,
    
    -- Gas & fees
    default_gas_price_gwei DECIMAL(10,2) DEFAULT 30.0,
    max_gas_price_gwei DECIMAL(10,2) DEFAULT 100.0,
    priority_fee_gwei DECIMAL(10,2) DEFAULT 2.0,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_verified BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(chain_id, network_type)
);

CREATE INDEX IF NOT EXISTS idx_blockchain_networks_type ON blockchain_networks(network_type, is_active);

-- 2. Smart Contracts
CREATE TABLE IF NOT EXISTS smart_contracts (
    contract_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    protocol_id UUID, -- References protocols(id) - Note: Adjust if protocols table uses name as ID
    network_id UUID REFERENCES blockchain_networks(network_id),
    
    -- Contract details
    contract_address VARCHAR(42) NOT NULL,
    contract_name VARCHAR(200) NOT NULL,
    contract_type VARCHAR(50) NOT NULL,  -- 'reputation', 'governance', 'staking', 'market'
    contract_version VARCHAR(50) NOT NULL,
    abi JSONB,
    abi_hash VARCHAR(64) NOT NULL,
    
    -- Deployment
    deployer_address VARCHAR(42) NOT NULL,
    deployment_tx_hash VARCHAR(66) NOT NULL,
    deployment_block INTEGER NOT NULL,
    deployment_timestamp TIMESTAMPTZ NOT NULL,
    
    -- Verification
    is_verified BOOLEAN DEFAULT FALSE,
    verification_date TIMESTAMPTZ,
    verified_by VARCHAR(100),
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_paused BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(contract_address, network_id)
);

CREATE INDEX IF NOT EXISTS idx_smart_contracts_network ON smart_contracts(network_id, contract_address);

-- 3. Contract Events
CREATE TABLE IF NOT EXISTS contract_events (
    event_id BIGSERIAL PRIMARY KEY,
    contract_id UUID REFERENCES smart_contracts(contract_id),
    
    -- Event details
    event_name VARCHAR(100) NOT NULL,
    event_signature VARCHAR(66) NOT NULL,
    block_number INTEGER NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    log_index INTEGER NOT NULL,
    
    -- Event data
    event_data JSONB NOT NULL,
    topics TEXT[] NOT NULL,
    
    -- Sender/receiver
    sender_address VARCHAR(42),
    receiver_address VARCHAR(42),
    
    -- Value
    value_wei DECIMAL(30,0) DEFAULT 0,
    value_token DECIMAL(30,18) DEFAULT 0,
    
    -- Status
    is_processed BOOLEAN DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    processing_error TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    indexed_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_contract_events_processed ON contract_events(is_processed, created_at);

-- 4. Blockchain Transactions
CREATE TABLE IF NOT EXISTS blockchain_transactions (
    tx_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    network_id UUID REFERENCES blockchain_networks(network_id),
    contract_id UUID REFERENCES smart_contracts(contract_id),
    
    -- Transaction details
    tx_hash VARCHAR(66) NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42),
    block_number INTEGER,
    block_hash VARCHAR(66),
    
    -- Transaction data
    nonce INTEGER,
    gas_limit DECIMAL(30,0),
    gas_price_gwei DECIMAL(10,2),
    gas_used DECIMAL(30,0),
    effective_gas_price_gwei DECIMAL(10,2),
    
    -- Value & fees
    value_wei DECIMAL(30,0) DEFAULT 0,
    max_fee_per_gas_gwei DECIMAL(10,2),
    max_priority_fee_per_gas_gwei DECIMAL(10,2),
    total_fee_wei DECIMAL(30,0),
    
    -- Status
    status VARCHAR(20) DEFAULT 'pending',  -- 'pending', 'confirmed', 'failed'
    confirmations INTEGER DEFAULT 0,
    is_error BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ,
    mined_at TIMESTAMPTZ,
    
    UNIQUE(tx_hash)
);

-- 5. Token Standards
CREATE TABLE IF NOT EXISTS token_standards (
    token_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    protocol_id UUID,
    contract_id UUID REFERENCES smart_contracts(contract_id),
    
    -- Token details
    token_address VARCHAR(42) NOT NULL,
    token_name VARCHAR(100) NOT NULL,
    token_symbol VARCHAR(10) NOT NULL,
    token_decimals INTEGER DEFAULT 18,
    token_type VARCHAR(20) NOT NULL,  -- 'ERC20', 'ERC721', 'ERC1155'
    total_supply DECIMAL(30,18) DEFAULT 0,
    
    -- Metadata
    token_metadata JSONB DEFAULT '{}',
    metadata_uri VARCHAR(500),
    
    -- Distribution
    is_mintable BOOLEAN DEFAULT FALSE,
    is_burnable BOOLEAN DEFAULT TRUE,
    max_supply DECIMAL(30,18),
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_paused BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(token_address, contract_id)
);

-- 6. Token Balances
CREATE TABLE IF NOT EXISTS token_balances (
    balance_id BIGSERIAL PRIMARY KEY,
    token_id UUID REFERENCES token_standards(token_id),
    agent_id UUID NOT NULL, -- References agents(id)
    
    -- Balance details
    wallet_address VARCHAR(42) NOT NULL,
    balance DECIMAL(30,18) NOT NULL DEFAULT 0,
    locked_balance DECIMAL(30,18) DEFAULT 0,
    staked_balance DECIMAL(30,18) DEFAULT 0,
    
    -- Historical tracking
    balance_at_block INTEGER,
    last_transfer_tx_hash VARCHAR(66),
    last_transfer_block INTEGER,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Timestamps
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    snapshot_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(token_id, agent_id, wallet_address)
);

-- 7. Wallet Connections
CREATE TABLE IF NOT EXISTS wallet_connections (
    connection_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL,
    protocol_id UUID,
    
    -- Wallet details
    wallet_address VARCHAR(42) NOT NULL,
    wallet_type VARCHAR(50) NOT NULL,  -- 'eoa', 'smart_contract', 'multisig'
    chain_id INTEGER NOT NULL,
    
    -- Connection details
    connection_method VARCHAR(50) NOT NULL,  -- 'signature', 'transaction', 'delegate'
    signature_data JSONB,
    signed_message TEXT,
    
    -- Permissions
    permissions JSONB DEFAULT '{"read": true, "write": false, "admin": false}',
    scopes TEXT[] DEFAULT ARRAY['reputation:read'],
    
    -- Status
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    verified_at TIMESTAMPTZ,
    
    -- Timestamps
    connected_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(agent_id, wallet_address, chain_id)
);

-- 8. Cross-Chain Bridges
CREATE TABLE IF NOT EXISTS cross_chain_bridges (
    bridge_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Bridge details
    bridge_name VARCHAR(100) NOT NULL,
    source_network_id UUID REFERENCES blockchain_networks(network_id),
    target_network_id UUID REFERENCES blockchain_networks(network_id),
    
    -- Bridge contracts
    source_bridge_address VARCHAR(42) NOT NULL,
    target_bridge_address VARCHAR(42) NOT NULL,
    
    -- Token mapping
    source_token_address VARCHAR(42),
    target_token_address VARCHAR(42),
    
    -- Bridge parameters
    min_transfer_amount DECIMAL(30,18) DEFAULT 0,
    max_transfer_amount DECIMAL(30,18),
    bridge_fee_percent DECIMAL(5,2) DEFAULT 0.1,
    estimated_bridge_time_seconds INTEGER DEFAULT 300,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_paused BOOLEAN DEFAULT FALSE,
    
    -- Security
    requires_verification BOOLEAN DEFAULT TRUE,
    max_daily_volume DECIMAL(30,18),
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(source_network_id, target_network_id, source_token_address)
);

-- 9. Bridge Transactions
CREATE TABLE IF NOT EXISTS bridge_transactions (
    bridge_tx_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bridge_id UUID REFERENCES cross_chain_bridges(bridge_id),
    agent_id UUID,
    
    -- Transaction details
    source_tx_hash VARCHAR(66) NOT NULL,
    target_tx_hash VARCHAR(66),
    source_amount DECIMAL(30,18) NOT NULL,
    target_amount DECIMAL(30,18),
    
    -- Bridge status
    bridge_status VARCHAR(50) DEFAULT 'initiated',  -- 'initiated', 'confirmed', 'bridging', 'completed', 'failed'
    bridge_fee DECIMAL(30,18) DEFAULT 0,
    
    -- Timestamps
    initiated_at TIMESTAMPTZ DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ,
    bridged_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    UNIQUE(source_tx_hash)
);

-- 10. Oracle Feeds
CREATE TABLE IF NOT EXISTS oracle_feeds (
    oracle_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    protocol_id UUID,
    network_id UUID REFERENCES blockchain_networks(network_id),
    
    -- Oracle details
    oracle_address VARCHAR(42) NOT NULL,
    oracle_name VARCHAR(100) NOT NULL,
    feed_type VARCHAR(50) NOT NULL,  -- 'price', 'reputation', 'data', 'randomness'
    
    -- Feed parameters
    feed_data JSONB NOT NULL,
    update_interval_seconds INTEGER DEFAULT 3600,
    heartbeat_seconds INTEGER DEFAULT 86400,
    
    -- Accuracy tracking
    accuracy_score DECIMAL(5,4) DEFAULT 0.5,
    last_accuracy_update TIMESTAMPTZ,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    last_update TIMESTAMPTZ,
    last_value DECIMAL(30,18),
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(oracle_address, network_id, feed_type)
);

-- 11. Oracle Updates
CREATE TABLE IF NOT EXISTS oracle_updates (
    update_id BIGSERIAL PRIMARY KEY,
    oracle_id UUID REFERENCES oracle_feeds(oracle_id),
    
    -- Update details
    update_round INTEGER NOT NULL,
    update_value DECIMAL(30,18) NOT NULL,
    update_data JSONB,
    
    -- Source
    submitted_by VARCHAR(42),
    submission_tx_hash VARCHAR(66),
    
    -- Verification
    is_verified BOOLEAN DEFAULT FALSE,
    verification_data JSONB,
    
    -- Timestamps
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    effective_from TIMESTAMPTZ NOT NULL,
    effective_to TIMESTAMPTZ,
    
    UNIQUE(oracle_id, update_round)
);

-- Insert default blockchain networks
INSERT INTO blockchain_networks 
(network_name, chain_id, network_type, rpc_endpoint, wss_endpoint, explorer_url, is_active, is_verified)
VALUES 
('Ethereum Mainnet', 1, 'mainnet', 'https://eth-mainnet.g.alchemy.com/v2/demo', 'wss://eth-mainnet.g.alchemy.com/v2/demo', 'https://etherscan.io', TRUE, TRUE),
('Polygon Mainnet', 137, 'mainnet', 'https://polygon-mainnet.g.alchemy.com/v2/demo', 'wss://polygon-mainnet.g.alchemy.com/v2/demo', 'https://polygonscan.com', TRUE, TRUE),
('Arbitrum One', 42161, 'layer2', 'https://arb1.arbitrum.io/rpc', 'wss://arb1.arbitrum.io/ws', 'https://arbiscan.io', TRUE, TRUE),
('Optimism', 10, 'layer2', 'https://mainnet.optimism.io', 'wss://ws-mainnet.optimism.io', 'https://optimistic.etherscan.io', TRUE, TRUE),
('Base', 8453, 'layer2', 'https://mainnet.base.org', 'wss://mainnet.base.org', 'https://basescan.org', TRUE, TRUE),
('Ethereum Sepolia', 11155111, 'testnet', 'https://sepolia.infura.io/v3/demo', 'wss://sepolia.infura.io/ws/v3/demo', 'https://sepolia.etherscan.io', TRUE, TRUE)
ON CONFLICT (chain_id, network_type) DO NOTHING;
