/// Sequential versioned migration items for database schema evolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationItem {
    pub sequence: u32,
    pub version: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub filename: &'static str,
    pub sql: &'static str,
}

/// Returns the ordered sequence of migrations for MySQL
pub fn mysql_migrations() -> Vec<MigrationItem> {
    vec![
        MigrationItem {
            sequence: 1,
            version: "v0.1.0",
            name: "baseline",
            description: "Core platform tables (systems, configs, models, fields, records, admins, audit_logs)",
            filename: "0001_v0.1.0_baseline.sql",
            sql: include_str!("../migrations/mysql/versions/0001_v0.1.0_baseline.sql"),
        },
        MigrationItem {
            sequence: 2,
            version: "v0.2.0",
            name: "add_applications",
            description: "Add applications table for client application authentication and scoped API RBAC",
            filename: "0002_v0.2.0_add_applications.sql",
            sql: include_str!("../migrations/mysql/versions/0002_v0.2.0_add_applications.sql"),
        },
    ]
}

/// Returns the ordered sequence of migrations for PostgreSQL
pub fn postgres_migrations() -> Vec<MigrationItem> {
    vec![
        MigrationItem {
            sequence: 1,
            version: "v0.1.0",
            name: "baseline",
            description: "Core platform tables (systems, configs, models, fields, records, admins, audit_logs)",
            filename: "0001_v0.1.0_baseline.sql",
            sql: include_str!("../migrations/postgres/versions/0001_v0.1.0_baseline.sql"),
        },
        MigrationItem {
            sequence: 2,
            version: "v0.2.0",
            name: "add_applications",
            description: "Add applications table for client application authentication and scoped API RBAC",
            filename: "0002_v0.2.0_add_applications.sql",
            sql: include_str!("../migrations/postgres/versions/0002_v0.2.0_add_applications.sql"),
        },
    ]
}

/// Returns the ordered sequence of migrations for MariaDB
pub fn mariadb_migrations() -> Vec<MigrationItem> {
    vec![
        MigrationItem {
            sequence: 1,
            version: "v0.1.0",
            name: "baseline",
            description: "Core platform tables (systems, configs, models, fields, records, admins, audit_logs)",
            filename: "0001_v0.1.0_baseline.sql",
            sql: include_str!("../migrations/mariadb/versions/0001_v0.1.0_baseline.sql"),
        },
        MigrationItem {
            sequence: 2,
            version: "v0.2.0",
            name: "add_applications",
            description: "Add applications table for client application authentication and scoped API RBAC",
            filename: "0002_v0.2.0_add_applications.sql",
            sql: include_str!("../migrations/mariadb/versions/0002_v0.2.0_add_applications.sql"),
        },
    ]
}

/// Returns the ordered sequence of migrations for MongoDB
pub fn mongodb_migrations() -> Vec<MigrationItem> {
    vec![
        MigrationItem {
            sequence: 1,
            version: "v0.1.0",
            name: "baseline",
            description: "Core platform collections and indexes",
            filename: "0001_v0.1.0_baseline.js",
            sql: include_str!("../migrations/mongodb/versions/0001_v0.1.0_baseline.js"),
        },
        MigrationItem {
            sequence: 2,
            version: "v0.2.0",
            name: "add_applications",
            description: "Add applications collection and indexes for client application authentication",
            filename: "0002_v0.2.0_add_applications.js",
            sql: include_str!("../migrations/mongodb/versions/0002_v0.2.0_add_applications.js"),
        },
    ]
}

/// Returns migrations for the given dialect string
pub fn get_migrations_for_dialect(dialect: &str) -> Vec<MigrationItem> {
    match dialect.to_ascii_lowercase().as_str() {
        "mysql" => mysql_migrations(),
        "mariadb" => mariadb_migrations(),
        "mongodb" => mongodb_migrations(),
        _ => postgres_migrations(),
    }
}
