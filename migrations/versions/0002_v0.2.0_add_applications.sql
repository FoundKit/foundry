-- ============================================================================
-- Foundry Migration: v0.2.0_add_applications (PostgreSQL)
-- Upstream Framework: Foundry v0.2.0
-- Target Engine: PostgreSQL 14+
-- Description: Add applications table for client application authentication and scoped API RBAC
-- ============================================================================

CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id VARCHAR(32) NOT NULL,
    name VARCHAR(64) NOT NULL,
    app_key VARCHAR(64) NOT NULL UNIQUE,
    app_secret VARCHAR(128) NOT NULL,
    description TEXT,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_applications_system_id ON applications(system_id);
CREATE INDEX IF NOT EXISTS idx_applications_app_key ON applications(app_key);
CREATE INDEX IF NOT EXISTS idx_applications_app_secret ON applications(app_secret);
