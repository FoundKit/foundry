-- ============================================================================
-- FOUNDRY ORM AUTO-GENERATED DDL MIGRATION (PostgreSQL)
-- Target Database Dialect: PostgreSQL
-- Generated directly from code-defined ORM schemas
-- ============================================================================

-- Enable required PostgreSQL extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- [1. systems] Core system and topic registry table
CREATE TABLE IF NOT EXISTS systems (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    slug VARCHAR(32) UNIQUE NOT NULL,
    name VARCHAR(64) NOT NULL,
    description VARCHAR(255) NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX IF NOT EXISTS idx_systems_slug ON systems(slug);
CREATE INDEX IF NOT EXISTS idx_systems_name ON systems(name);
CREATE INDEX IF NOT EXISTS idx_systems_status ON systems(status);
CREATE INDEX IF NOT EXISTS idx_systems_created_at ON systems(created_at);

-- [2. system_configs] System configuration entries with UI widget options
CREATE TABLE IF NOT EXISTS system_configs (
    id BIGSERIAL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,
    key VARCHAR(64) NOT NULL,
    label VARCHAR(64) NOT NULL,
    value_type VARCHAR(24) NOT NULL,
    value JSONB NULL,
    options JSONB NOT NULL DEFAULT '{}'::jsonb,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_system_configs_key ON system_configs(system_id, key);
CREATE INDEX IF NOT EXISTS idx_system_configs_tenant ON system_configs(system_id, sort_order);

-- [3. models] Dynamic data model metadata definitions
CREATE TABLE IF NOT EXISTS models (
    id BIGSERIAL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,
    slug VARCHAR(48) NOT NULL,
    name VARCHAR(64) NOT NULL,
    description VARCHAR(255) NULL,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_models_system_slug ON models(system_id, slug);

-- [4. model_fields] Model field schemas with validation and rendering rules
CREATE TABLE IF NOT EXISTS model_fields (
    id BIGSERIAL PRIMARY KEY,
    model_id BIGINT NOT NULL,
    name VARCHAR(48) NOT NULL,
    label VARCHAR(64) NOT NULL,
    field_type VARCHAR(32) NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    default_value JSONB NULL,
    options JSONB NOT NULL DEFAULT '{}'::jsonb,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_model_fields_name ON model_fields(model_id, name);
CREATE INDEX IF NOT EXISTS idx_model_fields_sort ON model_fields(model_id, sort_order);

-- [5. model_records] Universal multi-record entity table (Mode A: Zero-DDL)
CREATE TABLE IF NOT EXISTS model_records (
    id BIGSERIAL PRIMARY KEY,
    model_id BIGINT NOT NULL DEFAULT 0,
    system_id VARCHAR(32) NOT NULL,
    model_slug VARCHAR(48) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX IF NOT EXISTS idx_model_records_lookup ON model_records(system_id, model_slug, created_at);
CREATE INDEX IF NOT EXISTS idx_model_records_query ON model_records(model_id, deleted_at, created_at, id);
CREATE INDEX IF NOT EXISTS idx_model_records_data_gin ON model_records USING GIN(data);

-- [6. admins] Unified admin identities with multi-tenant RBAC permissions
CREATE TABLE IF NOT EXISTS admins (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    username VARCHAR(48) UNIQUE NOT NULL,
    email VARCHAR(96) UNIQUE NULL,
    password_hash VARCHAR(128) NOT NULL,
    role VARCHAR(24) NOT NULL DEFAULT 'topic_admin',
    allowed_systems JSONB NOT NULL DEFAULT '[]'::jsonb,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX IF NOT EXISTS idx_admins_username ON admins(username);
CREATE INDEX IF NOT EXISTS idx_admins_role ON admins(role);
CREATE INDEX IF NOT EXISTS idx_admins_status ON admins(status);

-- [7. audit_logs] Admin non-GET request audit log trail
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    admin_id UUID NULL,
    admin_username VARCHAR(48) NULL,
    system_slug VARCHAR(32) NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(255) NOT NULL,
    action_name VARCHAR(64) NULL,
    headers JSONB NOT NULL DEFAULT '{}'::jsonb,
    query_params VARCHAR(2048) NULL,
    body_params TEXT NULL,
    ip_address VARCHAR(45) NULL,
    user_agent VARCHAR(512) NULL,
    status_code SMALLINT NULL,
    duration_ms INTEGER NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_audit_logs_admin_id ON audit_logs(admin_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_system_slug ON audit_logs(system_slug, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_path ON audit_logs(path, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);

-- [8. applications] Client applications with scoped API and model CRUD permissions
CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    system_id VARCHAR(32) NOT NULL,
    name VARCHAR(64) NOT NULL,
    token VARCHAR(128) UNIQUE NOT NULL,
    description VARCHAR(255) NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX IF NOT EXISTS idx_applications_system_id ON applications(system_id);
CREATE INDEX IF NOT EXISTS idx_applications_token ON applications(token);

-- ============================================================================
-- AUTOMATIC TIMESTAMP TRIGGER
-- ============================================================================
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_timestamp_systems ON systems;
CREATE TRIGGER set_timestamp_systems BEFORE UPDATE ON systems FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
DROP TRIGGER IF EXISTS set_timestamp_system_configs ON system_configs;
CREATE TRIGGER set_timestamp_system_configs BEFORE UPDATE ON system_configs FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
DROP TRIGGER IF EXISTS set_timestamp_models ON models;
CREATE TRIGGER set_timestamp_models BEFORE UPDATE ON models FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
DROP TRIGGER IF EXISTS set_timestamp_model_fields ON model_fields;
CREATE TRIGGER set_timestamp_model_fields BEFORE UPDATE ON model_fields FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
DROP TRIGGER IF EXISTS set_timestamp_model_records ON model_records;
CREATE TRIGGER set_timestamp_model_records BEFORE UPDATE ON model_records FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
DROP TRIGGER IF EXISTS set_timestamp_admins ON admins;
CREATE TRIGGER set_timestamp_admins BEFORE UPDATE ON admins FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();
DROP TRIGGER IF EXISTS set_timestamp_applications ON applications;
CREATE TRIGGER set_timestamp_applications BEFORE UPDATE ON applications FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();

-- ============================================================================
-- DEFAULT SEED DATA (admin / admin123456)
-- ============================================================================
INSERT INTO admins (username, email, password_hash, role, allowed_systems)
VALUES (
    'admin',
    'admin@foundry.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q',
    'super_admin',
    '["*"]'::jsonb
) ON CONFLICT (username) DO NOTHING;
