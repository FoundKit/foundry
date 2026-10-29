-- ============================================================================
-- Foundry Migration: v0.2.0_add_applications (MySQL)
-- Upstream Framework: Foundry v0.2.0
-- Target Engine: MySQL 8.0+
-- Description: Add applications table for client application authentication and scoped API RBAC
-- Safe for manual DBA execution with self-recording metadata tracking
-- ============================================================================

CREATE TABLE IF NOT EXISTS applications (
    id BINARY(16) NOT NULL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,
    name VARCHAR(64) NOT NULL,
    app_key VARCHAR(64) NOT NULL UNIQUE,
    app_secret VARCHAR(128) NOT NULL,
    description TEXT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    permissions JSON NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    INDEX idx_applications_system_id (system_id),
    INDEX idx_applications_app_key (app_key),
    INDEX idx_applications_app_secret (app_secret)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================================
-- Migration Metadata Tracking (Records sequence 2 into _foundry_migrations)
-- ============================================================================
CREATE TABLE IF NOT EXISTS `_foundry_migrations` (
    `sequence` INT NOT NULL PRIMARY KEY,
    `version` VARCHAR(32) NOT NULL,
    `name` VARCHAR(64) NOT NULL,
    `applied_at` DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO `_foundry_migrations` (`sequence`, `version`, `name`, `applied_at`)
VALUES (2, 'v0.2.0', 'add_applications', CURRENT_TIMESTAMP(6))
ON DUPLICATE KEY UPDATE `applied_at` = CURRENT_TIMESTAMP(6);
