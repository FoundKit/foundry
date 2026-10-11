use crate::orm::dialect::Dialect;
use foundry_core::orm::{ColumnDef, ColumnType, DefaultValue, EntitySchema};

/// Compiles one or more EntitySchemas into a complete SQL DDL migration script for the target dialect.
pub fn compile_schemas(schemas: &[EntitySchema], dialect: Dialect) -> String {
    let mut out = String::new();

    let dialect_name = match dialect {
        Dialect::Postgres => "PostgreSQL",
        Dialect::MySql => "MySQL",
        Dialect::MariaDb => "MariaDB",
    };

    out.push_str("-- ============================================================================\n");
    out.push_str(&format!(
        "-- FOUNDRY ORM AUTO-GENERATED DDL MIGRATION ({})\n",
        dialect_name
    ));
    out.push_str(&format!("-- Target Database Dialect: {}\n", dialect_name));
    out.push_str("-- Generated directly from code-defined ORM schemas\n");
    out.push_str("-- ============================================================================\n\n");

    if dialect == Dialect::Postgres {
        out.push_str("-- Enable required PostgreSQL extensions\n");
        out.push_str("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";\n");
        out.push_str("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\";\n\n");
    }

    for (i, schema) in schemas.iter().enumerate() {
        out.push_str(&format!(
            "-- [{}. {}] {}\n",
            i + 1,
            schema.table_name,
            schema.comment.as_deref().unwrap_or("")
        ));
        out.push_str(&compile_schema(schema, dialect));
        out.push_str("\n\n");
    }

    if dialect == Dialect::Postgres {
        // Automatic timestamp trigger for postgres
        let tables_with_updated_at: Vec<&str> = schemas
            .iter()
            .filter(|s| s.columns.iter().any(|c| c.name == "updated_at"))
            .map(|s| s.table_name.as_str())
            .collect();

        if !tables_with_updated_at.is_empty() {
            out.push_str("-- ============================================================================\n");
            out.push_str("-- AUTOMATIC TIMESTAMP TRIGGER\n");
            out.push_str("-- ============================================================================\n");
            out.push_str("CREATE OR REPLACE FUNCTION trigger_set_timestamp()\n");
            out.push_str("RETURNS TRIGGER AS $$\n");
            out.push_str("BEGIN\n");
            out.push_str("    NEW.updated_at = NOW();\n");
            out.push_str("    RETURN NEW;\n");
            out.push_str("END;\n");
            out.push_str("$$ LANGUAGE plpgsql;\n\n");

            for table in tables_with_updated_at {
                out.push_str(&format!(
                    "DROP TRIGGER IF EXISTS set_timestamp_{} ON {};\n",
                    table, table
                ));
                out.push_str(&format!(
                    "CREATE TRIGGER set_timestamp_{} BEFORE UPDATE ON {} FOR EACH ROW EXECUTE FUNCTION trigger_set_timestamp();\n",
                    table, table
                ));
            }
            out.push('\n');
        }
    }

    // Default Seed Admin if `admins` table exists
    if schemas.iter().any(|s| s.table_name == "admins") {
        out.push_str("-- ============================================================================\n");
        out.push_str("-- DEFAULT SEED DATA (admin / admin123456)\n");
        out.push_str("-- ============================================================================\n");
        match dialect {
            Dialect::Postgres => {
                out.push_str(
                    r#"INSERT INTO admins (username, email, password_hash, role, allowed_systems)
VALUES (
    'admin',
    'admin@foundry.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q',
    'super_admin',
    '["*"]'::jsonb
) ON CONFLICT (username) DO NOTHING;
"#,
                );
            }
            Dialect::MySql | Dialect::MariaDb => {
                out.push_str(
                    r#"INSERT IGNORE INTO admins (id, username, email, password_hash, role, allowed_systems)
VALUES (
    UNHEX(REPLACE('00000000-0000-0000-0000-000000000001', '-', '')),
    'admin',
    'admin@foundry.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KX9WKigtvygJxZkV8V0k5w$Nf0fZMa6tRQWnFTqVO1xlFyFO/fzcvM1lmZ6hwnlD7Q',
    'super_admin',
    '["*"]'
);
"#,
                );
            }
        }
    }

    out
}

/// Compiles a single EntitySchema into SQL DDL for the target dialect.
pub fn compile_schema(schema: &EntitySchema, dialect: Dialect) -> String {
    match dialect {
        Dialect::Postgres => compile_pg_schema(schema),
        Dialect::MySql | Dialect::MariaDb => compile_mysql_schema(schema),
    }
}

fn compile_pg_schema(schema: &EntitySchema) -> String {
    let mut out = String::new();
    out.push_str(&format!("CREATE TABLE IF NOT EXISTS {} (\n", schema.table_name));

    let mut col_lines = Vec::new();
    for col in &schema.columns {
        let mut def = format!("    {} {}", col.name, pg_type(col));

        if col.is_primary_key {
            def.push_str(" PRIMARY KEY");
        } else if col.is_unique {
            def.push_str(" UNIQUE");
        }

        if !col.is_auto_increment {
            if col.is_nullable {
                def.push_str(" NULL");
            } else {
                def.push_str(" NOT NULL");
            }
        }

        let def_val = pg_default_val(&col.default_value);
        if !def_val.is_empty() {
            def.push(' ');
            def.push_str(&def_val);
        }

        col_lines.push(def);
    }

    out.push_str(&col_lines.join(",\n"));
    out.push_str("\n);");

    // Indexes
    for idx in &schema.indexes {
        out.push('\n');
        if idx.is_gin {
            out.push_str(&format!(
                "CREATE INDEX IF NOT EXISTS {} ON {} USING GIN({});",
                idx.name,
                schema.table_name,
                idx.columns.join(", ")
            ));
        } else if idx.is_unique {
            out.push_str(&format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {}({});",
                idx.name,
                schema.table_name,
                idx.columns.join(", ")
            ));
        } else {
            out.push_str(&format!(
                "CREATE INDEX IF NOT EXISTS {} ON {}({});",
                idx.name,
                schema.table_name,
                idx.columns.join(", ")
            ));
        }
    }

    out
}

fn compile_mysql_schema(schema: &EntitySchema) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "CREATE TABLE IF NOT EXISTS `{}` (\n",
        schema.table_name
    ));

    let mut body_lines = Vec::new();
    let mut pk_cols = Vec::new();

    for col in &schema.columns {
        if col.is_primary_key {
            pk_cols.push(format!("`{}`", col.name));
        }

        let mut def = format!("    `{}` {}", col.name, mysql_type(schema, col));

        if !col.is_nullable {
            def.push_str(" NOT NULL");
        } else {
            def.push_str(" NULL");
        }

        if col.is_auto_increment {
            def.push_str(" AUTO_INCREMENT");
        }

        let def_val = mysql_default_val(col);
        if !def_val.is_empty() {
            def.push(' ');
            def.push_str(&def_val);
        }

        body_lines.push(def);
    }

    if !pk_cols.is_empty() {
        body_lines.push(format!("    PRIMARY KEY ({})", pk_cols.join(", ")));
    }

    for idx in &schema.indexes {
        if idx.is_gin {
            // MySQL does not support GIN indexes
            continue;
        }
        let cols = idx
            .columns
            .iter()
            .map(|c| format!("`{}`", c))
            .collect::<Vec<_>>()
            .join(", ");
        if idx.is_unique {
            body_lines.push(format!("    UNIQUE KEY `{}` ({})", idx.name, cols));
        } else {
            body_lines.push(format!("    INDEX `{}` ({})", idx.name, cols));
        }
    }

    out.push_str(&body_lines.join(",\n"));
    out.push_str("\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;");

    out
}

fn pg_type(col: &ColumnDef) -> &'static str {
    match &col.col_type {
        ColumnType::Uuid => "UUID",
        ColumnType::BigInt => {
            if col.is_auto_increment {
                "BIGSERIAL"
            } else {
                "BIGINT"
            }
        }
        ColumnType::Integer => {
            if col.is_auto_increment {
                "SERIAL"
            } else {
                "INTEGER"
            }
        }
        ColumnType::SmallInt => "SMALLINT",
        ColumnType::VarChar(n) => match n {
            10 => "VARCHAR(10)",
            20 => "VARCHAR(20)",
            24 => "VARCHAR(24)",
            32 => "VARCHAR(32)",
            45 => "VARCHAR(45)",
            48 => "VARCHAR(48)",
            50 => "VARCHAR(50)",
            64 => "VARCHAR(64)",
            96 => "VARCHAR(96)",
            100 => "VARCHAR(100)",
            128 => "VARCHAR(128)",
            255 => "VARCHAR(255)",
            512 => "VARCHAR(512)",
            2048 => "VARCHAR(2048)",
            _ => "VARCHAR(255)",
        },
        ColumnType::Text => "TEXT",
        ColumnType::Json => "JSONB",
        ColumnType::Boolean => "BOOLEAN",
        ColumnType::DateTime => "TIMESTAMPTZ",
        ColumnType::Decimal(_, _) => "DECIMAL(10, 2)",
        ColumnType::Float => "DOUBLE PRECISION",
    }
}

fn pg_default_val(val: &DefaultValue) -> String {
    match val {
        DefaultValue::None => String::new(),
        DefaultValue::UuidV4 => "DEFAULT gen_random_uuid()".to_string(),
        DefaultValue::CurrentTimestamp => "DEFAULT NOW()".to_string(),
        DefaultValue::EmptyJson => "DEFAULT '{}'::jsonb".to_string(),
        DefaultValue::EmptyJsonArray => "DEFAULT '[]'::jsonb".to_string(),
        DefaultValue::SmallInt(i) => format!("DEFAULT {}", i),
        DefaultValue::Integer(i) => format!("DEFAULT {}", i),
        DefaultValue::BigInt(i) => format!("DEFAULT {}", i),
        DefaultValue::Boolean(b) => format!("DEFAULT {}", if *b { "TRUE" } else { "FALSE" }),
        DefaultValue::String(s) => format!("DEFAULT '{}'", s),
    }
}

fn mysql_type(schema: &EntitySchema, col: &ColumnDef) -> &'static str {
    match &col.col_type {
        ColumnType::Uuid => {
            if schema.table_name == "admins" || col.name == "admin_id" || col.name == "id" {
                "BINARY(16)"
            } else {
                "CHAR(36)"
            }
        }
        ColumnType::BigInt => "BIGINT",
        ColumnType::Integer => "INT",
        ColumnType::SmallInt => "SMALLINT",
        ColumnType::VarChar(n) => match n {
            10 => "VARCHAR(10)",
            20 => "VARCHAR(20)",
            24 => "VARCHAR(24)",
            32 => "VARCHAR(32)",
            45 => "VARCHAR(45)",
            48 => "VARCHAR(48)",
            50 => "VARCHAR(50)",
            64 => "VARCHAR(64)",
            96 => "VARCHAR(96)",
            100 => "VARCHAR(100)",
            128 => "VARCHAR(128)",
            255 => "VARCHAR(255)",
            512 => "VARCHAR(512)",
            2048 => "VARCHAR(2048)",
            _ => "VARCHAR(255)",
        },
        ColumnType::Text => "TEXT",
        ColumnType::Json => "JSON",
        ColumnType::Boolean => "BOOLEAN",
        ColumnType::DateTime => "DATETIME(6)",
        ColumnType::Decimal(_, _) => "DECIMAL(10, 2)",
        ColumnType::Float => "DOUBLE",
    }
}

fn mysql_default_val(col: &ColumnDef) -> String {
    if col.name == "updated_at" && col.col_type == ColumnType::DateTime {
        return "DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6)".to_string();
    }
    match &col.default_value {
        DefaultValue::None => String::new(),
        DefaultValue::UuidV4 => String::new(),
        DefaultValue::CurrentTimestamp => "DEFAULT CURRENT_TIMESTAMP(6)".to_string(),
        DefaultValue::EmptyJson | DefaultValue::EmptyJsonArray => String::new(),
        DefaultValue::SmallInt(i) => format!("DEFAULT {}", i),
        DefaultValue::Integer(i) => format!("DEFAULT {}", i),
        DefaultValue::BigInt(i) => format!("DEFAULT {}", i),
        DefaultValue::Boolean(b) => format!("DEFAULT {}", if *b { 1 } else { 0 }),
        DefaultValue::String(s) => format!("DEFAULT '{}'", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orm::models::core_platform_models;

    #[test]
    fn test_compile_postgres_ddl() {
        let models = core_platform_models();
        let ddl = compile_schemas(&models, Dialect::Postgres);
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS systems"));
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS admins"));
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS model_records"));
        assert!(ddl.contains("trigger_set_timestamp"));
        assert!(ddl.contains("super_admin"));
    }

    #[test]
    fn test_compile_mysql_ddl() {
        let models = core_platform_models();
        let ddl = compile_schemas(&models, Dialect::MySql);
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS `systems`"));
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS `admins`"));
        assert!(ddl.contains("ENGINE=InnoDB"));
        assert!(ddl.contains("super_admin"));
    }
}
