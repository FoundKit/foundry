use foundry_core::orm::{ColumnDef, DefaultValue, EntitySchema, IndexDef};

/// Returns the schema for `systems` platform table
pub fn systems_schema() -> EntitySchema {
    EntitySchema::new("systems")
        .column(ColumnDef::uuid("id").primary_key().default_uuid())
        .column(ColumnDef::varchar("slug", 32).not_null().unique())
        .column(ColumnDef::varchar("name", 64).not_null())
        .column(ColumnDef::varchar("description", 255).nullable())
        .column(
            ColumnDef::smallint("status")
                .not_null()
                .default_val(DefaultValue::SmallInt(1)),
        )
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .column(ColumnDef::datetime("updated_at").default_current_timestamp())
        .column(ColumnDef::datetime("deleted_at").nullable())
        .index(IndexDef::new("idx_systems_slug", &["slug"]))
        .index(IndexDef::new("idx_systems_name", &["name"]))
        .index(IndexDef::new("idx_systems_status", &["status"]))
        .index(IndexDef::new("idx_systems_created_at", &["created_at"]))
        .comment("Core system and topic registry table")
}

/// Returns the schema for `system_configs` platform table
pub fn system_configs_schema() -> EntitySchema {
    EntitySchema::new("system_configs")
        .column(ColumnDef::bigint("id").primary_key().auto_increment())
        .column(ColumnDef::varchar("system_id", 32).not_null())
        .column(ColumnDef::varchar("key", 64).not_null())
        .column(ColumnDef::varchar("label", 64).not_null())
        .column(ColumnDef::varchar("value_type", 24).not_null())
        .column(ColumnDef::json("value").nullable())
        .column(ColumnDef::json("options").default_empty_json())
        .column(
            ColumnDef::integer("sort_order")
                .not_null()
                .default_val(DefaultValue::Integer(0)),
        )
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .column(ColumnDef::datetime("updated_at").default_current_timestamp())
        .column(ColumnDef::datetime("deleted_at").nullable())
        .index(IndexDef::new("uk_system_configs_key", &["system_id", "key"]).unique())
        .index(IndexDef::new(
            "idx_system_configs_tenant",
            &["system_id", "sort_order"],
        ))
        .comment("System configuration entries with UI widget options")
}

/// Returns the schema for `models` platform table
pub fn models_schema() -> EntitySchema {
    EntitySchema::new("models")
        .column(ColumnDef::bigint("id").primary_key().auto_increment())
        .column(ColumnDef::varchar("system_id", 32).not_null())
        .column(ColumnDef::varchar("slug", 48).not_null())
        .column(ColumnDef::varchar("name", 64).not_null())
        .column(ColumnDef::varchar("description", 255).nullable())
        .column(
            ColumnDef::boolean("is_system")
                .not_null()
                .default_val(DefaultValue::Boolean(false)),
        )
        .column(
            ColumnDef::smallint("status")
                .not_null()
                .default_val(DefaultValue::SmallInt(1)),
        )
        .column(ColumnDef::json("permissions").default_empty_json())
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .column(ColumnDef::datetime("updated_at").default_current_timestamp())
        .column(ColumnDef::datetime("deleted_at").nullable())
        .index(IndexDef::new("uk_models_system_slug", &["system_id", "slug"]).unique())
        .comment("Dynamic data model metadata definitions")
}

/// Returns the schema for `model_fields` platform table
pub fn model_fields_schema() -> EntitySchema {
    EntitySchema::new("model_fields")
        .column(ColumnDef::bigint("id").primary_key().auto_increment())
        .column(ColumnDef::bigint("model_id").not_null())
        .column(ColumnDef::varchar("name", 48).not_null())
        .column(ColumnDef::varchar("label", 64).not_null())
        .column(ColumnDef::varchar("field_type", 32).not_null())
        .column(
            ColumnDef::boolean("is_required")
                .not_null()
                .default_val(DefaultValue::Boolean(false)),
        )
        .column(ColumnDef::json("default_value").nullable())
        .column(ColumnDef::json("options").default_empty_json())
        .column(
            ColumnDef::integer("sort_order")
                .not_null()
                .default_val(DefaultValue::Integer(0)),
        )
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .column(ColumnDef::datetime("updated_at").default_current_timestamp())
        .column(ColumnDef::datetime("deleted_at").nullable())
        .index(IndexDef::new("uk_model_fields_name", &["model_id", "name"]).unique())
        .index(IndexDef::new(
            "idx_model_fields_sort",
            &["model_id", "sort_order"],
        ))
        .comment("Model field schemas with validation and rendering rules")
}

/// Returns the schema for `model_records` platform table
pub fn model_records_schema() -> EntitySchema {
    EntitySchema::new("model_records")
        .column(ColumnDef::bigint("id").primary_key().auto_increment())
        .column(
            ColumnDef::bigint("model_id")
                .not_null()
                .default_val(DefaultValue::BigInt(0)),
        )
        .column(ColumnDef::varchar("system_id", 32).not_null())
        .column(ColumnDef::varchar("model_slug", 48).not_null())
        .column(ColumnDef::json("data").default_empty_json())
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .column(ColumnDef::datetime("updated_at").default_current_timestamp())
        .column(ColumnDef::datetime("deleted_at").nullable())
        .index(IndexDef::new(
            "idx_model_records_lookup",
            &["system_id", "model_slug", "created_at"],
        ))
        .index(IndexDef::new(
            "idx_model_records_query",
            &["model_id", "deleted_at", "created_at", "id"],
        ))
        .index(IndexDef::new("idx_model_records_data_gin", &["data"]).gin())
        .comment("Universal multi-record entity table (Mode A: Zero-DDL)")
}

/// Returns the schema for `admins` platform table
pub fn admins_schema() -> EntitySchema {
    EntitySchema::new("admins")
        .column(ColumnDef::uuid("id").primary_key().default_uuid())
        .column(ColumnDef::varchar("username", 48).not_null().unique())
        .column(ColumnDef::varchar("email", 96).nullable().unique())
        .column(ColumnDef::varchar("password_hash", 128).not_null())
        .column(
            ColumnDef::varchar("role", 24)
                .not_null()
                .default_val(DefaultValue::String("topic_admin".into())),
        )
        .column(ColumnDef::json("allowed_systems").default_empty_json_array())
        .column(
            ColumnDef::smallint("status")
                .not_null()
                .default_val(DefaultValue::SmallInt(1)),
        )
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .column(ColumnDef::datetime("updated_at").default_current_timestamp())
        .column(ColumnDef::datetime("deleted_at").nullable())
        .index(IndexDef::new("idx_admins_username", &["username"]))
        .index(IndexDef::new("idx_admins_role", &["role"]))
        .index(IndexDef::new("idx_admins_status", &["status"]))
        .comment("Unified admin identities with multi-tenant RBAC permissions")
}

/// Returns the schema for `audit_logs` platform table
pub fn audit_logs_schema() -> EntitySchema {
    EntitySchema::new("audit_logs")
        .column(ColumnDef::bigint("id").primary_key().auto_increment())
        .column(ColumnDef::uuid("admin_id").nullable())
        .column(ColumnDef::varchar("admin_username", 48).nullable())
        .column(ColumnDef::varchar("system_slug", 32).nullable())
        .column(ColumnDef::varchar("method", 10).not_null())
        .column(ColumnDef::varchar("path", 255).not_null())
        .column(ColumnDef::varchar("action_name", 64).nullable())
        .column(ColumnDef::json("headers").default_empty_json())
        .column(ColumnDef::varchar("query_params", 2048).nullable())
        .column(ColumnDef::text("body_params").nullable())
        .column(ColumnDef::varchar("ip_address", 45).nullable())
        .column(ColumnDef::varchar("user_agent", 512).nullable())
        .column(ColumnDef::smallint("status_code").nullable())
        .column(ColumnDef::integer("duration_ms").nullable())
        .column(ColumnDef::datetime("created_at").default_current_timestamp())
        .index(IndexDef::new(
            "idx_audit_logs_admin_id",
            &["admin_id", "created_at"],
        ))
        .index(IndexDef::new(
            "idx_audit_logs_system_slug",
            &["system_slug", "created_at"],
        ))
        .index(IndexDef::new(
            "idx_audit_logs_path",
            &["path", "created_at"],
        ))
        .index(IndexDef::new("idx_audit_logs_created_at", &["created_at"]))
        .comment("Admin non-GET request audit log trail")
}

/// Returns all 7 core platform ORM schemas in dependency order
pub fn core_platform_models() -> Vec<EntitySchema> {
    vec![
        systems_schema(),
        system_configs_schema(),
        models_schema(),
        model_fields_schema(),
        model_records_schema(),
        admins_schema(),
        audit_logs_schema(),
    ]
}
