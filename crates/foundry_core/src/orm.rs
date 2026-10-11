use serde::{Deserialize, Serialize};

/// Supported column data types in the Foundry ORM model system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    Uuid,
    BigInt,
    Integer,
    SmallInt,
    VarChar(u32),
    Text,
    Json,
    Boolean,
    DateTime,
    Decimal(u32, u32),
    Float,
}

/// Supported default column values in the Foundry ORM model system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefaultValue {
    None,
    UuidV4,
    CurrentTimestamp,
    EmptyJson,
    EmptyJsonArray,
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Boolean(bool),
    String(String),
}

/// Column definition within an ORM Entity schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: ColumnType,
    pub is_primary_key: bool,
    pub is_auto_increment: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub default_value: DefaultValue,
    pub comment: Option<String>,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            col_type,
            is_primary_key: false,
            is_auto_increment: false,
            is_nullable: false,
            is_unique: false,
            default_value: DefaultValue::None,
            comment: None,
        }
    }

    pub fn uuid(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::Uuid)
    }

    pub fn bigint(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::BigInt)
    }

    pub fn integer(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::Integer)
    }

    pub fn smallint(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::SmallInt)
    }

    pub fn varchar(name: impl Into<String>, len: u32) -> Self {
        Self::new(name, ColumnType::VarChar(len))
    }

    pub fn text(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::Text)
    }

    pub fn json(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::Json)
    }

    pub fn boolean(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::Boolean)
    }

    pub fn datetime(name: impl Into<String>) -> Self {
        Self::new(name, ColumnType::DateTime)
    }

    pub fn decimal(name: impl Into<String>, precision: u32, scale: u32) -> Self {
        Self::new(name, ColumnType::Decimal(precision, scale))
    }

    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_nullable = false;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.is_auto_increment = true;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.is_nullable = false;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.is_nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn default_uuid(mut self) -> Self {
        self.default_value = DefaultValue::UuidV4;
        self
    }

    pub fn default_current_timestamp(mut self) -> Self {
        self.default_value = DefaultValue::CurrentTimestamp;
        self
    }

    pub fn default_empty_json(mut self) -> Self {
        self.default_value = DefaultValue::EmptyJson;
        self
    }

    pub fn default_empty_json_array(mut self) -> Self {
        self.default_value = DefaultValue::EmptyJsonArray;
        self
    }

    pub fn default_val(mut self, val: DefaultValue) -> Self {
        self.default_value = val;
        self
    }
}

/// Index definition within an ORM Entity schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDef {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_gin: bool,
}

impl IndexDef {
    pub fn new(name: impl Into<String>, columns: &[&str]) -> Self {
        Self {
            name: name.into(),
            columns: columns.iter().map(|s| s.to_string()).collect(),
            is_unique: false,
            is_gin: false,
        }
    }

    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn gin(mut self) -> Self {
        self.is_gin = true;
        self
    }
}

/// Top-level Entity / Table schema in the Foundry ORM model system (Prisma-like)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySchema {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
    pub indexes: Vec<IndexDef>,
    pub comment: Option<String>,
}

impl EntitySchema {
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            columns: Vec::new(),
            indexes: Vec::new(),
            comment: None,
        }
    }

    pub fn column(mut self, col: ColumnDef) -> Self {
        self.columns.push(col);
        self
    }

    pub fn columns(mut self, cols: impl IntoIterator<Item = ColumnDef>) -> Self {
        self.columns.extend(cols);
        self
    }

    pub fn index(mut self, idx: IndexDef) -> Self {
        self.indexes.push(idx);
        self
    }

    pub fn indexes(mut self, idxs: impl IntoIterator<Item = IndexDef>) -> Self {
        self.indexes.extend(idxs);
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}
