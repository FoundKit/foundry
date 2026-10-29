-- ============================================================================
-- Foundry Migration: v0.2.0_add_applications (MariaDB)
-- Upstream Framework: Foundry v0.2.0
-- Target Engine: MariaDB 10.6+
-- Description: Add applications table for client application authentication and scoped API RBAC
-- ============================================================================

CREATE TABLE IF NOT EXISTS applications (
    id UUID NOT NULL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,
    name VARCHAR(64) NOT NULL,
    app_key VARCHAR(64) NOT NULL UNIQUE,
    app_secret VARCHAR(128) NOT NULL,
    description TEXT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions LONGTEXT NOT NULL CHECK (json_valid(permissions)),
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    INDEX idx_applications_system_id (system_id),
    INDEX idx_applications_app_key (app_key),
    INDEX idx_applications_app_secret (app_secret)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
