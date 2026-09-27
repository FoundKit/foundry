-- ============================================================================
-- FOUNDRY PLATFORM - INITIAL DATABASE SCHEMA MIGRATION (MySQL & MariaDB)
-- Architecture: Universal Single-Table Engine with Covering Index
-- ============================================================================

CREATE TABLE IF NOT EXISTS systems (
    id BINARY(16) NOT NULL PRIMARY KEY,
    slug VARCHAR(32) NOT NULL UNIQUE,
    name VARCHAR(64) NOT NULL,
    description VARCHAR(255) NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    INDEX idx_systems_slug (slug),
    INDEX idx_systems_name (name),
    INDEX idx_systems_status (status),
    INDEX idx_systems_created_at (created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS system_configs (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,
    `key` VARCHAR(64) NOT NULL,
    label VARCHAR(64) NOT NULL,
    value_type VARCHAR(24) NOT NULL,
    value JSON NULL,
    options JSON NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    UNIQUE KEY uk_system_configs_key (system_id, `key`),
    INDEX idx_system_configs_tenant (system_id, sort_order ASC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS models (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,
    slug VARCHAR(48) NOT NULL,
    name VARCHAR(64) NOT NULL,
    description VARCHAR(255) NULL,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions JSON NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    UNIQUE KEY uk_models_system_slug (system_id, slug),
    INDEX idx_models_tenant (system_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS model_fields (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    model_id BIGINT NOT NULL,
    name VARCHAR(48) NOT NULL,
    label VARCHAR(64) NOT NULL,
    field_type VARCHAR(24) NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    default_value JSON NULL,
    options JSON NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    UNIQUE KEY uk_model_fields_model_name (model_id, name),
    INDEX idx_model_fields_sort (model_id, sort_order ASC),
    CONSTRAINT fk_model_fields_model FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================================
-- Universal Single-Table model_records
-- ============================================================================
CREATE TABLE IF NOT EXISTS model_records (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    model_id BIGINT NOT NULL,
    system_id VARCHAR(32) NOT NULL,
    model_slug VARCHAR(48) NOT NULL,
    data JSON NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    INDEX idx_model_query (model_id, deleted_at, created_at DESC, id DESC),
    INDEX idx_system_query (system_id, deleted_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS admins (
    id BINARY(16) NOT NULL PRIMARY KEY,
    username VARCHAR(48) NOT NULL UNIQUE,
    email VARCHAR(96) NULL UNIQUE,
    password_hash VARCHAR(128) NOT NULL,
    role VARCHAR(24) NOT NULL DEFAULT 'topic_admin',
    allowed_systems JSON NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    INDEX idx_admins_username (username),
    INDEX idx_admins_role (role),
    INDEX idx_admins_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    admin_id BINARY(16) NULL,
    admin_username VARCHAR(48) NULL,
    system_slug VARCHAR(32) NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(255) NOT NULL,
    action_name VARCHAR(64) NULL,
    headers JSON NOT NULL,
    query_params VARCHAR(2048) NULL,
    body_params MEDIUMTEXT NULL,
    ip_address VARCHAR(45) NULL,
    user_agent VARCHAR(512) NULL,
    status_code SMALLINT NULL,
    duration_ms INT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    INDEX idx_audit_logs_admin_id (admin_id, created_at DESC),
    INDEX idx_audit_logs_system_slug (system_slug, created_at DESC),
    INDEX idx_audit_logs_path (path, created_at DESC),
    INDEX idx_audit_logs_created_at (created_at DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================================
-- DEFAULT INITIAL SUPER ADMIN (admin / admin123456)
-- ============================================================================
INSERT IGNORE INTO admins (id, username, email, password_hash, role, allowed_systems)
VALUES (
    UNHEX(REPLACE('00000000-0000-0000-0000-000000000001', '-', '')),
    'admin',
    'admin@foundry.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q',
    'super_admin',
    '["*"]'
);
