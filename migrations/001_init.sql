-- Event Store Schema for Inventory Management System
-- Generated: 2026-01-11

-- Events table: Stores all domain events
CREATE TABLE events (
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    sequence BIGINT NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_version VARCHAR(20) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

-- Index for efficient event loading by aggregate
CREATE INDEX idx_events_aggregate ON events(aggregate_type, aggregate_id);

-- Index for time-based queries
CREATE INDEX idx_events_timestamp ON events(timestamp);

-- Snapshots table: For future optimization (aggregate state caching)
CREATE TABLE snapshots (
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    last_sequence BIGINT NOT NULL,
    payload JSONB NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (aggregate_type, aggregate_id)
);

-- Index for snapshot queries
CREATE INDEX idx_snapshots_timestamp ON snapshots(timestamp);
