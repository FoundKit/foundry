use serde::{Deserialize, Serialize};

/// Supported field types in Foundry Zero-DDL dynamic schema & system configs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    Richtext,
    Image,
    File,
    Integer,
    Number,
    Boolean,
    Datetime,
    Array,
    Relation,
}

impl FieldType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Richtext => "richtext",
            Self::Image => "image",
            Self::File => "file",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Datetime => "datetime",
            Self::Array => "array",
            Self::Relation => "relation",
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for FieldType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "string" => Ok(Self::String),
            "richtext" => Ok(Self::Richtext),
            "image" => Ok(Self::Image),
            "file" => Ok(Self::File),
            "integer" => Ok(Self::Integer),
            "number" | "float" => Ok(Self::Number),
            "boolean" | "bool" => Ok(Self::Boolean),
            "datetime" | "date" => Ok(Self::Datetime),
            "array" => Ok(Self::Array),
            "relation" => Ok(Self::Relation),
            other => Err(format!("Unknown field type: {}", other)),
        }
    }
}

/// System status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum SystemStatus {
    Disabled = 0,
    Active = 1,
    Archived = 2,
}

impl SystemStatus {
    pub fn from_i16(val: i16) -> Self {
        match val {
            0 => Self::Disabled,
            1 => Self::Active,
            2 => Self::Archived,
            _ => Self::Disabled,
        }
    }
}

/// Validate system_slug or model_slug format (`^[a-z0-9_-]{2,32}$` / `^[a-z0-9_-]{2,48}$`)
pub fn is_valid_slug(slug: &str, max_len: usize) -> bool {
    let len = slug.len();
    if !(2..=max_len).contains(&len) {
        return false;
    }
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_validation() {
        assert!(is_valid_slug("carnival_2026", 32));
        assert!(is_valid_slug("vip-mall", 32));
        assert!(is_valid_slug("ab", 32));
        assert!(!is_valid_slug("a", 32));
        assert!(!is_valid_slug("Carnival", 32)); // uppercase forbidden
        assert!(!is_valid_slug("carnival!", 32));
        assert!(!is_valid_slug(
            "this_slug_is_way_too_long_to_be_valid_under_limit",
            32
        ));
    }

    #[test]
    fn test_field_type_parsing_and_display() {
        assert_eq!("string".parse::<FieldType>().unwrap(), FieldType::String);
        assert_eq!(
            "richtext".parse::<FieldType>().unwrap(),
            FieldType::Richtext
        );
        assert_eq!("image".parse::<FieldType>().unwrap(), FieldType::Image);
        assert_eq!("file".parse::<FieldType>().unwrap(), FieldType::File);
        assert_eq!("integer".parse::<FieldType>().unwrap(), FieldType::Integer);
        assert_eq!("number".parse::<FieldType>().unwrap(), FieldType::Number);
        assert_eq!("float".parse::<FieldType>().unwrap(), FieldType::Number);
        assert_eq!("boolean".parse::<FieldType>().unwrap(), FieldType::Boolean);
        assert_eq!("bool".parse::<FieldType>().unwrap(), FieldType::Boolean);
        assert_eq!(
            "datetime".parse::<FieldType>().unwrap(),
            FieldType::Datetime
        );
        assert_eq!("array".parse::<FieldType>().unwrap(), FieldType::Array);
        assert_eq!(
            "relation".parse::<FieldType>().unwrap(),
            FieldType::Relation
        );
        assert!("unknown".parse::<FieldType>().is_err());

        assert_eq!(FieldType::String.to_string(), "string");
        assert_eq!(FieldType::Integer.as_str(), "integer");
    }

    #[test]
    fn test_system_status() {
        assert_eq!(SystemStatus::from_i16(0), SystemStatus::Disabled);
        assert_eq!(SystemStatus::from_i16(1), SystemStatus::Active);
        assert_eq!(SystemStatus::from_i16(2), SystemStatus::Archived);
        assert_eq!(SystemStatus::from_i16(99), SystemStatus::Disabled);
    }
}
