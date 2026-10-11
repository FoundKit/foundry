use crate::orm::dialect::Dialect;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Strongly-typed SQL parameter value for ORM operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrmValue {
    Null,
    Bool(bool),
    SmallInt(i16),
    Int(i32),
    BigInt(i64),
    Float(f64),
    String(String),
    Json(Value),
    Uuid(Uuid),
    DateTime(DateTime<Utc>),
}

impl From<bool> for OrmValue {
    fn from(v: bool) -> Self {
        OrmValue::Bool(v)
    }
}

impl From<i16> for OrmValue {
    fn from(v: i16) -> Self {
        OrmValue::SmallInt(v)
    }
}

impl From<i32> for OrmValue {
    fn from(v: i32) -> Self {
        OrmValue::Int(v)
    }
}

impl From<i64> for OrmValue {
    fn from(v: i64) -> Self {
        OrmValue::BigInt(v)
    }
}

impl From<u64> for OrmValue {
    fn from(v: u64) -> Self {
        OrmValue::BigInt(v as i64)
    }
}

impl From<f64> for OrmValue {
    fn from(v: f64) -> Self {
        OrmValue::Float(v)
    }
}

impl From<&str> for OrmValue {
    fn from(v: &str) -> Self {
        OrmValue::String(v.to_string())
    }
}

impl From<String> for OrmValue {
    fn from(v: String) -> Self {
        OrmValue::String(v)
    }
}

impl From<Value> for OrmValue {
    fn from(v: Value) -> Self {
        OrmValue::Json(v)
    }
}

impl From<Uuid> for OrmValue {
    fn from(v: Uuid) -> Self {
        OrmValue::Uuid(v)
    }
}

impl From<DateTime<Utc>> for OrmValue {
    fn from(v: DateTime<Utc>) -> Self {
        OrmValue::DateTime(v)
    }
}

impl<T: Into<OrmValue>> From<Option<T>> for OrmValue {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(inner) => inner.into(),
            None => OrmValue::Null,
        }
    }
}

/// Filter condition for queries
#[derive(Debug, Clone, PartialEq)]
pub enum OrmCondition {
    Eq(String, OrmValue),
    Ne(String, OrmValue),
    IsNull(String),
    IsNotNull(String),
    In(String, Vec<OrmValue>),
    And(Vec<OrmCondition>),
    Or(Vec<OrmCondition>),
    Raw(String, Vec<OrmValue>),
}

fn render_condition(cond: &OrmCondition, dialect: Dialect, binds: &mut Vec<OrmValue>) -> String {
    match cond {
        OrmCondition::Eq(col, val) => {
            let quoted = if col.contains('.') {
                col.clone()
            } else {
                dialect.quote_ident(col)
            };
            let ph = dialect.placeholder(binds.len() + 1);
            binds.push(val.clone());
            format!("{} = {}", quoted, ph)
        }
        OrmCondition::Ne(col, val) => {
            let quoted = if col.contains('.') {
                col.clone()
            } else {
                dialect.quote_ident(col)
            };
            let ph = dialect.placeholder(binds.len() + 1);
            binds.push(val.clone());
            format!("{} != {}", quoted, ph)
        }
        OrmCondition::IsNull(col) => {
            let quoted = if col.contains('.') {
                col.clone()
            } else {
                dialect.quote_ident(col)
            };
            format!("{} IS NULL", quoted)
        }
        OrmCondition::IsNotNull(col) => {
            let quoted = if col.contains('.') {
                col.clone()
            } else {
                dialect.quote_ident(col)
            };
            format!("{} IS NOT NULL", quoted)
        }
        OrmCondition::In(col, vals) => {
            let quoted = if col.contains('.') {
                col.clone()
            } else {
                dialect.quote_ident(col)
            };
            if vals.is_empty() {
                "1 = 0".to_string()
            } else {
                let mut phs = Vec::new();
                for v in vals {
                    phs.push(dialect.placeholder(binds.len() + 1));
                    binds.push(v.clone());
                }
                format!("{} IN ({})", quoted, phs.join(", "))
            }
        }
        OrmCondition::And(conds) => {
            let parts: Vec<String> = conds
                .iter()
                .map(|c| render_condition(c, dialect, binds))
                .collect();
            format!("({})", parts.join(" AND "))
        }
        OrmCondition::Or(conds) => {
            let parts: Vec<String> = conds
                .iter()
                .map(|c| render_condition(c, dialect, binds))
                .collect();
            format!("({})", parts.join(" OR "))
        }
        OrmCondition::Raw(sql, vals) => {
            for v in vals {
                binds.push(v.clone());
            }
            sql.clone()
        }
    }
}

/// Typed ORM SELECT query builder
#[derive(Debug, Clone)]
pub struct OrmSelect {
    table: String,
    columns: Vec<String>,
    conditions: Vec<OrmCondition>,
    order_by: Vec<(String, bool)>, // (column, is_desc)
    limit: Option<i64>,
    offset: Option<i64>,
    is_count: bool,
}

impl OrmSelect {
    pub fn from_table(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            is_count: false,
        }
    }

    pub fn column(mut self, col: impl Into<String>) -> Self {
        self.columns.push(col.into());
        self
    }

    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns.extend(cols.iter().map(|s| s.to_string()));
        self
    }

    pub fn count(mut self) -> Self {
        self.is_count = true;
        self
    }

    pub fn where_eq(mut self, col: impl Into<String>, val: impl Into<OrmValue>) -> Self {
        self.conditions.push(OrmCondition::Eq(col.into(), val.into()));
        self
    }

    pub fn where_ne(mut self, col: impl Into<String>, val: impl Into<OrmValue>) -> Self {
        self.conditions.push(OrmCondition::Ne(col.into(), val.into()));
        self
    }

    pub fn where_null(mut self, col: impl Into<String>) -> Self {
        self.conditions.push(OrmCondition::IsNull(col.into()));
        self
    }

    pub fn where_not_null(mut self, col: impl Into<String>) -> Self {
        self.conditions.push(OrmCondition::IsNotNull(col.into()));
        self
    }

    pub fn where_in(
        mut self,
        col: impl Into<String>,
        vals: impl IntoIterator<Item = impl Into<OrmValue>>,
    ) -> Self {
        self.conditions.push(OrmCondition::In(
            col.into(),
            vals.into_iter().map(|v| v.into()).collect(),
        ));
        self
    }

    pub fn filter(mut self, cond: OrmCondition) -> Self {
        self.conditions.push(cond);
        self
    }

    pub fn order_by_asc(mut self, col: impl Into<String>) -> Self {
        self.order_by.push((col.into(), false));
        self
    }

    pub fn order_by_desc(mut self, col: impl Into<String>) -> Self {
        self.order_by.push((col.into(), true));
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn to_sql(&self, dialect: Dialect) -> (String, Vec<OrmValue>) {
        let mut sql = String::new();
        let mut binds = Vec::new();

        if self.is_count {
            sql.push_str(&format!(
                "SELECT COUNT(*) FROM {}",
                dialect.quote_ident(&self.table)
            ));
        } else {
            let cols = if self.columns.is_empty() {
                "*".to_string()
            } else {
                self.columns
                    .iter()
                    .map(|c| {
                        if c.contains('(') || c.contains('.') || c.contains('*') {
                            c.clone()
                        } else {
                            dialect.quote_ident(c)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            sql.push_str(&format!(
                "SELECT {} FROM {}",
                cols,
                dialect.quote_ident(&self.table)
            ));
        }

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut cond_strs = Vec::new();
            for cond in &self.conditions {
                cond_strs.push(render_condition(cond, dialect, &mut binds));
            }
            sql.push_str(&cond_strs.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_strs: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, is_desc)| {
                    let quoted = if col.contains('.') {
                        col.clone()
                    } else {
                        dialect.quote_ident(col)
                    };
                    if *is_desc {
                        format!("{} DESC", quoted)
                    } else {
                        format!("{} ASC", quoted)
                    }
                })
                .collect();
            sql.push_str(&order_strs.join(", "));
        }

        if let Some(limit) = self.limit {
            let placeholder = dialect.placeholder(binds.len() + 1);
            sql.push_str(&format!(" LIMIT {}", placeholder));
            binds.push(OrmValue::BigInt(limit));
        }

        if let Some(offset) = self.offset {
            let placeholder = dialect.placeholder(binds.len() + 1);
            sql.push_str(&format!(" OFFSET {}", placeholder));
            binds.push(OrmValue::BigInt(offset));
        }

        (sql, binds)
    }
}

/// Typed ORM INSERT query builder
#[derive(Debug, Clone)]
pub struct OrmInsert {
    table: String,
    values: Vec<(String, OrmValue)>,
    returning: Vec<String>,
}

impl OrmInsert {
    pub fn into_table(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            values: Vec::new(),
            returning: Vec::new(),
        }
    }

    pub fn value(mut self, col: impl Into<String>, val: impl Into<OrmValue>) -> Self {
        self.values.push((col.into(), val.into()));
        self
    }

    pub fn returning(mut self, cols: &[&str]) -> Self {
        self.returning.extend(cols.iter().map(|s| s.to_string()));
        self
    }

    pub fn to_sql(&self, dialect: Dialect) -> (String, Vec<OrmValue>) {
        let mut sql = String::new();
        let mut binds = Vec::new();

        let cols: Vec<String> = self
            .values
            .iter()
            .map(|(c, _)| dialect.quote_ident(c))
            .collect();
        let mut phs = Vec::new();
        for (_, val) in &self.values {
            phs.push(dialect.placeholder(binds.len() + 1));
            binds.push(val.clone());
        }

        sql.push_str(&format!(
            "INSERT INTO {} ({}) VALUES ({})",
            dialect.quote_ident(&self.table),
            cols.join(", "),
            phs.join(", ")
        ));

        if dialect == Dialect::Postgres && !self.returning.is_empty() {
            let ret_cols = self
                .returning
                .iter()
                .map(|c| dialect.quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            sql.push_str(&format!(" RETURNING {}", ret_cols));
        }

        (sql, binds)
    }
}

/// Typed ORM UPDATE query builder
#[derive(Debug, Clone)]
pub struct OrmUpdate {
    table: String,
    assignments: Vec<(String, OrmValue)>,
    set_now_fields: Vec<String>,
    conditions: Vec<OrmCondition>,
    returning: Vec<String>,
}

impl OrmUpdate {
    pub fn table(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            assignments: Vec::new(),
            set_now_fields: Vec::new(),
            conditions: Vec::new(),
            returning: Vec::new(),
        }
    }

    pub fn set(mut self, col: impl Into<String>, val: impl Into<OrmValue>) -> Self {
        self.assignments.push((col.into(), val.into()));
        self
    }

    pub fn set_now(mut self, col: impl Into<String>) -> Self {
        self.set_now_fields.push(col.into());
        self
    }

    pub fn where_eq(mut self, col: impl Into<String>, val: impl Into<OrmValue>) -> Self {
        self.conditions.push(OrmCondition::Eq(col.into(), val.into()));
        self
    }

    pub fn where_null(mut self, col: impl Into<String>) -> Self {
        self.conditions.push(OrmCondition::IsNull(col.into()));
        self
    }

    pub fn filter(mut self, cond: OrmCondition) -> Self {
        self.conditions.push(cond);
        self
    }

    pub fn returning(mut self, cols: &[&str]) -> Self {
        self.returning.extend(cols.iter().map(|s| s.to_string()));
        self
    }

    pub fn to_sql(&self, dialect: Dialect) -> (String, Vec<OrmValue>) {
        let mut sql = String::new();
        let mut binds = Vec::new();

        sql.push_str(&format!("UPDATE {} SET ", dialect.quote_ident(&self.table)));

        let mut set_parts = Vec::new();
        for (col, val) in &self.assignments {
            let ph = dialect.placeholder(binds.len() + 1);
            binds.push(val.clone());
            set_parts.push(format!("{} = {}", dialect.quote_ident(col), ph));
        }
        for col in &self.set_now_fields {
            set_parts.push(format!(
                "{} = {}",
                dialect.quote_ident(col),
                dialect.now_expr()
            ));
        }

        sql.push_str(&set_parts.join(", "));

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut cond_strs = Vec::new();
            for cond in &self.conditions {
                cond_strs.push(render_condition(cond, dialect, &mut binds));
            }
            sql.push_str(&cond_strs.join(" AND "));
        }

        if dialect == Dialect::Postgres && !self.returning.is_empty() {
            let ret_cols = self
                .returning
                .iter()
                .map(|c| dialect.quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            sql.push_str(&format!(" RETURNING {}", ret_cols));
        }

        (sql, binds)
    }
}

/// Typed ORM DELETE query builder
#[derive(Debug, Clone)]
pub struct OrmDelete {
    table: String,
    conditions: Vec<OrmCondition>,
}

impl OrmDelete {
    pub fn from_table(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            conditions: Vec::new(),
        }
    }

    pub fn where_eq(mut self, col: impl Into<String>, val: impl Into<OrmValue>) -> Self {
        self.conditions.push(OrmCondition::Eq(col.into(), val.into()));
        self
    }

    pub fn filter(mut self, cond: OrmCondition) -> Self {
        self.conditions.push(cond);
        self
    }

    pub fn to_sql(&self, dialect: Dialect) -> (String, Vec<OrmValue>) {
        let mut sql = String::new();
        let mut binds = Vec::new();

        sql.push_str(&format!("DELETE FROM {}", dialect.quote_ident(&self.table)));

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let mut cond_strs = Vec::new();
            for cond in &self.conditions {
                cond_strs.push(render_condition(cond, dialect, &mut binds));
            }
            sql.push_str(&cond_strs.join(" AND "));
        }

        (sql, binds)
    }
}

// ============================================================================
// SQLx Parameter Binding Helpers
// ============================================================================

#[cfg(feature = "postgres")]
pub fn bind_pg_query<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    values: Vec<OrmValue>,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for val in values {
        query = match val {
            OrmValue::Null => query.bind(None::<String>),
            OrmValue::Bool(b) => query.bind(b),
            OrmValue::SmallInt(i) => query.bind(i),
            OrmValue::Int(i) => query.bind(i),
            OrmValue::BigInt(i) => query.bind(i),
            OrmValue::Float(f) => query.bind(f),
            OrmValue::String(s) => query.bind(s),
            OrmValue::Json(j) => query.bind(j),
            OrmValue::Uuid(u) => query.bind(u),
            OrmValue::DateTime(dt) => query.bind(dt),
        };
    }
    query
}

#[cfg(feature = "postgres")]
pub fn bind_pg_query_as<'q, T>(
    mut query: sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments>,
    values: Vec<OrmValue>,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments> {
    for val in values {
        query = match val {
            OrmValue::Null => query.bind(None::<String>),
            OrmValue::Bool(b) => query.bind(b),
            OrmValue::SmallInt(i) => query.bind(i),
            OrmValue::Int(i) => query.bind(i),
            OrmValue::BigInt(i) => query.bind(i),
            OrmValue::Float(f) => query.bind(f),
            OrmValue::String(s) => query.bind(s),
            OrmValue::Json(j) => query.bind(j),
            OrmValue::Uuid(u) => query.bind(u),
            OrmValue::DateTime(dt) => query.bind(dt),
        };
    }
    query
}

#[cfg(any(feature = "mysql", feature = "mariadb"))]
pub fn bind_mysql_query<'q>(
    mut query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    values: Vec<OrmValue>,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    for val in values {
        query = match val {
            OrmValue::Null => query.bind(None::<String>),
            OrmValue::Bool(b) => query.bind(b),
            OrmValue::SmallInt(i) => query.bind(i),
            OrmValue::Int(i) => query.bind(i),
            OrmValue::BigInt(i) => query.bind(i),
            OrmValue::Float(f) => query.bind(f),
            OrmValue::String(s) => query.bind(s),
            OrmValue::Json(j) => query.bind(j),
            OrmValue::Uuid(u) => query.bind(u.to_string()),
            OrmValue::DateTime(dt) => query.bind(dt),
        };
    }
    query
}

#[cfg(any(feature = "mysql", feature = "mariadb"))]
pub fn bind_mysql_query_as<'q, T>(
    mut query: sqlx::query::QueryAs<'q, sqlx::MySql, T, sqlx::mysql::MySqlArguments>,
    values: Vec<OrmValue>,
) -> sqlx::query::QueryAs<'q, sqlx::MySql, T, sqlx::mysql::MySqlArguments> {
    for val in values {
        query = match val {
            OrmValue::Null => query.bind(None::<String>),
            OrmValue::Bool(b) => query.bind(b),
            OrmValue::SmallInt(i) => query.bind(i),
            OrmValue::Int(i) => query.bind(i),
            OrmValue::BigInt(i) => query.bind(i),
            OrmValue::Float(f) => query.bind(f),
            OrmValue::String(s) => query.bind(s),
            OrmValue::Json(j) => query.bind(j),
            OrmValue::Uuid(u) => query.bind(u.to_string()),
            OrmValue::DateTime(dt) => query.bind(dt),
        };
    }
    query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orm_select_postgres() {
        let query = OrmSelect::from_table("model_records")
            .columns(&["id", "data"])
            .where_eq("system_id", "sample")
            .where_null("deleted_at")
            .order_by_desc("created_at")
            .limit(20)
            .offset(0);

        let (sql, binds) = query.to_sql(Dialect::Postgres);
        assert_eq!(
            sql,
            "SELECT \"id\", \"data\" FROM \"model_records\" WHERE \"system_id\" = $1 AND \"deleted_at\" IS NULL ORDER BY \"created_at\" DESC LIMIT $2 OFFSET $3"
        );
        assert_eq!(binds.len(), 3);
        assert_eq!(binds[0], OrmValue::String("sample".to_string()));
        assert_eq!(binds[1], OrmValue::BigInt(20));
        assert_eq!(binds[2], OrmValue::BigInt(0));
    }

    #[test]
    fn test_orm_select_mysql() {
        let query = OrmSelect::from_table("model_records")
            .columns(&["id", "data"])
            .where_eq("system_id", "sample")
            .where_null("deleted_at")
            .limit(10);

        let (sql, binds) = query.to_sql(Dialect::MySql);
        assert_eq!(
            sql,
            "SELECT `id`, `data` FROM `model_records` WHERE `system_id` = ? AND `deleted_at` IS NULL LIMIT ?"
        );
        assert_eq!(binds.len(), 2);
    }

    #[test]
    fn test_orm_insert_postgres() {
        let query = OrmInsert::into_table("model_records")
            .value("system_id", "sample")
            .value("model_slug", "articles")
            .returning(&["id", "system_id"]);

        let (sql, binds) = query.to_sql(Dialect::Postgres);
        assert_eq!(
            sql,
            "INSERT INTO \"model_records\" (\"system_id\", \"model_slug\") VALUES ($1, $2) RETURNING \"id\", \"system_id\""
        );
        assert_eq!(binds.len(), 2);
    }

    #[test]
    fn test_orm_update_mysql() {
        let query = OrmUpdate::table("model_records")
            .set("data", serde_json::json!({"title": "Test"}))
            .set_now("updated_at")
            .where_eq("id", 123i64);

        let (sql, binds) = query.to_sql(Dialect::MySql);
        assert_eq!(
            sql,
            "UPDATE `model_records` SET `data` = ?, `updated_at` = NOW(6) WHERE `id` = ?"
        );
        assert_eq!(binds.len(), 2);
    }
}
