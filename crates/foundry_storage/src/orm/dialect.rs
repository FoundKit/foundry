use serde::{Deserialize, Serialize};

/// Supported SQL database dialects in Foundry ORM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dialect {
    Postgres,
    MySql,
    MariaDb,
}

impl Dialect {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Some(Dialect::Postgres),
            "mysql" => Some(Dialect::MySql),
            "mariadb" => Some(Dialect::MariaDb),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Dialect::Postgres => "postgres",
            Dialect::MySql => "mysql",
            Dialect::MariaDb => "mariadb",
        }
    }

    pub fn quote_ident(&self, ident: &str) -> String {
        match self {
            Dialect::Postgres => format!("\"{}\"", ident),
            Dialect::MySql | Dialect::MariaDb => format!("`{}`", ident),
        }
    }

    pub fn placeholder(&self, index: usize) -> String {
        match self {
            Dialect::Postgres => format!("${}", index),
            Dialect::MySql | Dialect::MariaDb => "?".to_string(),
        }
    }

    pub fn now_expr(&self) -> &'static str {
        match self {
            Dialect::Postgres => "NOW()",
            Dialect::MySql | Dialect::MariaDb => "NOW(6)",
        }
    }
}
