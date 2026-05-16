-- AGP-CORE Database Schema
-- PostgreSQL initialization script

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Agents table (main entity)
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    fingerprint VARCHAR(64) UNIQUE NOT NULL,
    agent_type VARCHAR(50) NOT NULL DEFAULT 'inference',
    model_hash VARCHAR(128),
    operator_id VARCHAR(64),
    
    -- Endocrine state (JSONB for flexibility)
    endocrine_state JSONB NOT NULL DEFAULT '{"levels": {}, "system_time": 0}',
    
    -- Computed metrics
    alignment FLOAT NOT NULL DEFAULT 1.0,
    privilege_level VARCHAR(50) NOT NULL DEFAULT 'standard',
    health_status VARCHAR(50) NOT NULL DEFAULT 'normal',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for agents
CREATE INDEX IF NOT EXISTS idx_agents_fingerprint ON agents(fingerprint);
CREATE INDEX IF NOT EXISTS idx_agents_type ON agents(agent_type);
CREATE INDEX IF NOT EXISTS idx_agents_health ON agents(health_status);
CREATE INDEX IF NOT EXISTS idx_agents_updated ON agents(updated_at);

-- Observations table (behavioral events)
CREATE TABLE IF NOT EXISTS observations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    stimulus_type VARCHAR(100) NOT NULL,
    strength FLOAT NOT NULL,
    hormones_affected JSONB NOT NULL DEFAULT '{}',
    observer_id UUID,
    protocol_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for observations
CREATE INDEX IF NOT EXISTS idx_observations_agent ON observations(agent_id);
CREATE INDEX IF NOT EXISTS idx_observations_type ON observations(stimulus_type);
CREATE INDEX IF NOT EXISTS idx_observations_created ON observations(created_at);

-- System parameters table
CREATE TABLE IF NOT EXISTS system_parameters (
    key VARCHAR(100) PRIMARY KEY,
    value FLOAT NOT NULL,
    description TEXT,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default system parameters
INSERT INTO system_parameters (key, value, description) VALUES
    ('homeostasis_baseline', 0.5, 'Target hormone level for homeostasis'),
    ('homeostasis_tolerance', 0.1, 'Acceptable deviation from baseline'),
    ('allostasis_adaptation_rate', 0.01, 'Rate of set-point adaptation'),
    ('circadian_amplitude', 0.15, 'Circadian rhythm amplitude'),
    ('decay_interval', 60, 'Seconds between decay cycles')
ON CONFLICT (key) DO NOTHING;

-- Protocols table (for Phase 2)
CREATE TABLE IF NOT EXISTS protocols (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    version VARCHAR(50) DEFAULT '1.0.0',
    is_active BOOLEAN DEFAULT TRUE,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default test protocol
INSERT INTO protocols (name, description) VALUES
    ('test_protocol', 'Default test protocol for development')
ON CONFLICT (name) DO NOTHING;

-- Function to update updated_at automatically
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
DROP TRIGGER IF EXISTS update_agents_updated_at ON agents;
CREATE TRIGGER update_agents_updated_at
    BEFORE UPDATE ON agents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_protocols_updated_at ON protocols;
CREATE TRIGGER update_protocols_updated_at
    BEFORE UPDATE ON protocols
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- View for agent summary
CREATE OR REPLACE VIEW agent_summary AS
SELECT 
    id,
    name,
    agent_type,
    alignment,
    health_status,
    privilege_level,
    (endocrine_state->>'levels')::jsonb->>'cortisol' as cortisol,
    (endocrine_state->>'levels')::jsonb->>'oxytocin' as oxytocin,
    (endocrine_state->>'levels')::jsonb->>'dopamine' as dopamine,
    created_at,
    updated_at
FROM agents;
