-- ============================================================================
-- Foundry Migration: v0.2.0_add_applications (PostgreSQL)
-- Upstream Framework: Foundry v0.2.0
-- Target Engine: PostgreSQL 14+
-- Description: Add applications table for client application authentication and scoped API RBAC
-- Safe for manual DBA execution with self-recording metadata tracking
-- ============================================================================

CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id VARCHAR(32) NOT NULL,
    name VARCHAR(64) NOT NULL,
    token VARCHAR(128) NOT NULL UNIQUE,
    description TEXT,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_applications_system_id ON applications(system_id);
CREATE INDEX IF NOT EXISTS idx_applications_token ON applications(token);

-- ============================================================================
-- Migration Metadata Tracking (Records sequence 2 into _foundry_migrations)
-- ============================================================================
CREATE TABLE IF NOT EXISTS _foundry_migrations (
    sequence INT PRIMARY KEY,
    version VARCHAR(32) NOT NULL,
    name VARCHAR(64) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO _foundry_migrations (sequence, version, name, applied_at)
VALUES (2, 'v0.2.0', 'add_applications', CURRENT_TIMESTAMP)
ON CONFLICT (sequence) DO UPDATE SET applied_at = CURRENT_TIMESTAMP;
