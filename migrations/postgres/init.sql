-- ============================================================================
-- FOUNDRY PLATFORM - INITIAL DATABASE SCHEMA MIGRATION (MODE A: ZERO-DDL)
-- Organization: foundkit (https://github.com/foundkit)
-- Database: PostgreSQL 18.6+
-- ============================================================================

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- 1. CORE SYSTEM / TOPIC REGISTRY (专题/子系统核心注册表)
-- ============================================================================

CREATE TABLE IF NOT EXISTS systems (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(32) UNIQUE NOT NULL,             -- Unique immutable slug (e.g. 'carnival_2026')
    name VARCHAR(64) NOT NULL,                     -- Display name
    description VARCHAR(255),                      -- Short description
    status SMALLINT NOT NULL DEFAULT 1,            -- 1: Active, 0: Disabled, 2: Archived
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_systems_slug ON systems(slug) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_systems_name ON systems(name) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_systems_status ON systems(status);
CREATE INDEX IF NOT EXISTS idx_systems_created_at ON systems(created_at DESC);

-- ============================================================================
-- 2. SYSTEM CONFIGS (专题专属配置项表: 纯粹的配置键值与UI控件定义)
-- 替代冗余的轻量单例模型，直接存储专题全局配置(如头图、活动规则、时间、开关)
-- ============================================================================

CREATE TABLE IF NOT EXISTS system_configs (
    id BIGSERIAL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,                -- Scoped to Sub-System slug
    key VARCHAR(64) NOT NULL,                      -- Configuration Key (e.g. 'banner', 'rules_html', 'start_time')
    label VARCHAR(64) NOT NULL,                    -- Display label in visual Admin UI
    value_type VARCHAR(24) NOT NULL,               -- 'string', 'richtext', 'image', 'file', 'integer', 'number', 'boolean', 'datetime', 'array'
    value JSONB NULL,                              -- Typed configuration value (e.g. "https://...", 50, true, ["a","b"])
    options JSONB NOT NULL DEFAULT '{}'::jsonb,    -- Widget options (e.g. { "upload_provider": "qiniu", "min": 0, "max": 100 })
    sort_order INTEGER NOT NULL DEFAULT 0,         -- Form display order in settings page
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_system_configs_key UNIQUE (system_id, key)
);

CREATE INDEX IF NOT EXISTS idx_system_configs_tenant ON system_configs(system_id, sort_order ASC);

-- ============================================================================
-- 3. DYNAMIC DATA MODELS (海量多记录业务数据模型元数据表)
-- ============================================================================

CREATE TABLE IF NOT EXISTS models (
    id BIGSERIAL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,                -- Scoped to Sub-System slug
    slug VARCHAR(48) NOT NULL,                     -- Model identifier (e.g. 'products', 'articles')
    name VARCHAR(64) NOT NULL,                     -- Display name (e.g. '商品列表', '文章列表')
    description VARCHAR(255),                      -- Short description
    is_system BOOLEAN NOT NULL DEFAULT FALSE,      -- System built-in vs user created
    status SMALLINT NOT NULL DEFAULT 1,            -- 1: Active, 0: Disabled
    permissions JSONB NOT NULL DEFAULT '{"public_read": false, "public_write": false, "auth_read": true, "auth_write": false}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL,
    CONSTRAINT uk_models_system_slug UNIQUE (system_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_models_tenant ON models(system_id) WHERE deleted_at IS NULL;

-- ============================================================================
-- 4. MODEL FIELDS & FORM UI WIDGET SPECIFICATION (数据模型字段定义与控件配置表)
-- ============================================================================

CREATE TABLE IF NOT EXISTS model_fields (
    id BIGSERIAL PRIMARY KEY,
    model_id BIGINT NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    name VARCHAR(48) NOT NULL,                     -- Field key (e.g. 'price', 'title', 'cover_image')
    label VARCHAR(64) NOT NULL,                    -- Display label in visual Admin UI
    field_type VARCHAR(24) NOT NULL,               -- 'string', 'richtext', 'image', 'file', 'integer', 'number', 'boolean', 'datetime', 'array', 'relation'
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    default_value JSONB NULL,                      -- Default value representation
    options JSONB NOT NULL DEFAULT '{}'::jsonb,    -- Widget configurations & validation rules
    sort_order INTEGER NOT NULL DEFAULT 0,         -- Form field display order
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_model_fields_model_name UNIQUE (model_id, name)
);

CREATE INDEX IF NOT EXISTS idx_model_fields_sort ON model_fields(model_id, sort_order ASC);

-- ============================================================================
-- 5. MODEL RECORDS (数据模型海量记录存储表 - Mode A 多记录实体存储)
-- ============================================================================

CREATE TABLE IF NOT EXISTS model_records (
    id BIGSERIAL PRIMARY KEY,
    system_id VARCHAR(32) NOT NULL,                -- Sub-system slug
    model_slug VARCHAR(48) NOT NULL,               -- Data Model slug
    data JSONB NOT NULL DEFAULT '{}'::jsonb,       -- Full record payload
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Fast lookup by sub-system, model, and time range
CREATE INDEX IF NOT EXISTS idx_model_records_lookup ON model_records(system_id, model_slug, created_at DESC) WHERE deleted_at IS NULL;
-- GIN inverted index for arbitrary JSON property search & filtering
CREATE INDEX IF NOT EXISTS idx_model_records_data_gin ON model_records USING GIN(data);

-- ============================================================================
-- ============================================================================
-- 6. UNIFIED ADMIN IDENTITIES & THREE-TIER RBAC (平台分级管理员与专题授权表)
-- 包含超级管理员 (super_admin)、普通管理员 (admin)、专题管理员 (topic_admin)
-- - 超级管理员 (super_admin): 平台所有权限，allowed_systems 为 '["*"]'
-- - 普通管理员 (admin): 除了“管理员与权限管理”外的其他所有平台与子系统权限，allowed_systems 为 '["*"]'
-- - 专题管理员 (topic_admin): 平台概览基础模块及指定授权专题工作台权限，allowed_systems 为具体专题列表如 '["carnival_2026"]'
-- ============================================================================

CREATE TABLE IF NOT EXISTS admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(48) UNIQUE NOT NULL,          -- Username
    email VARCHAR(96) UNIQUE,                      -- Email
    password_hash VARCHAR(128) NOT NULL,           -- Argon2id password hash (~96-100 chars)
    role VARCHAR(24) NOT NULL DEFAULT 'topic_admin', -- 'super_admin' (超管) | 'admin' (普通管理员) | 'topic_admin' (专题管理员)
    allowed_systems JSONB NOT NULL DEFAULT '[]'::jsonb, -- 授权子系统 slug 列表 (超管与普通管理员为 '["*"]'；专题管理员由超管赋予特定专题如 '["carnival_2026", "vip_mall"]')
    status SMALLINT NOT NULL DEFAULT 1,            -- 1: Active, 0: Disabled
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_admins_username ON admins(username);
CREATE INDEX IF NOT EXISTS idx_admins_role ON admins(role);
CREATE INDEX IF NOT EXISTS idx_admins_status ON admins(status);

-- ============================================================================
-- 7. ADMIN NON-GET OPERATION AUDIT LOGS (管理员全量写操作/非GET请求审计日志表)
-- 记录管理员所有非 GET 请求（POST, PUT, PATCH, DELETE 等），分字段独立原始存储请求头、Query 字符串、Body 载荷与动态操作名
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    admin_id UUID NULL,                            -- 操作管理员 ID (未登录/登录中可为 NULL)
    admin_username VARCHAR(48) NULL,               -- 操作管理员用户名 (冗余字段便于快速展示与历史溯源)
    system_slug VARCHAR(32) NULL,                  -- 关联的专题/子系统 slug (全局平台级操作为 NULL)
    method VARCHAR(10) NOT NULL,                   -- HTTP 请求方法: POST, PUT, PATCH, DELETE 等 (记录管理员所有非 GET 请求)
    path VARCHAR(255) NOT NULL,                    -- 请求路径 (例如: /admin/auth/login, /admin/s/carnival_2026/models)
    action_name VARCHAR(64) NULL,                  -- 操作名称 (程序端根据路由与业务灵活映射增减，如 "登录", "创建数据模型", "修改专题配置", "删除数据记录")
    headers JSONB NOT NULL DEFAULT '{}'::jsonb,    -- 请求头 Header (键值对 JSONB，例如 Content-Type, X-Request-ID 等)
    query_params VARCHAR(2048) NULL,               -- 原始 Query 字符串 (例如: "page=1&limit=10" 或 "?page=1&limit=10", 2KB 充足冗余)
    body_params TEXT NULL,                         -- 原始请求体 Body (原始文本存储，支持 JSON、表单、纯文本等任意格式)
    ip_address VARCHAR(45) NULL,                   -- 客户端 IP 地址 (IPv4 / IPv6 标准最大 45 字符)
    user_agent VARCHAR(512) NULL,                  -- 客户端 User-Agent (512 字符充足冗余)
    status_code SMALLINT NULL,                     -- HTTP 响应状态码 (如 200, 400, 500, SMALLINT 2字节足够)
    duration_ms INTEGER NULL,                      -- 请求执行耗时 (毫秒)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()  -- 操作记录时间
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_admin_id ON audit_logs(admin_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_system_slug ON audit_logs(system_slug, created_at DESC) WHERE system_slug IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_path ON audit_logs(path, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- ============================================================================
-- 8. AUTOMATIC TIMESTAMP TRIGGER (自动更新 updated_at 触发器)
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

-- ============================================================================
-- 9. DEFAULT INITIAL SEED DATA (系统初始化超级管理员账号: admin / admin123456)
-- 拥有全量子系统管理权限 (allowed_systems = '["*"]')
-- Password Hash: Argon2id representation for 'admin123456'
-- ============================================================================

INSERT INTO admins (username, email, password_hash, role, allowed_systems)
VALUES (
    'admin',
    'admin@foundry.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q',
    'super_admin',
    '["*"]'::jsonb
) ON CONFLICT (username) DO NOTHING;
