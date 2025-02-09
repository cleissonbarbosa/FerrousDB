use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    Text(String),
    Integer(i64),
    Boolean(bool),
}

impl DataType {
    pub fn get_type(&self) -> &'static str {
        match self {
            DataType::Integer(_) => "INTEGER",
            DataType::Text(_) => "TEXT",
            DataType::Boolean(_) => "BOOLEAN",
        }
    }

    pub fn get_value(&self) -> String {
        match self {
            DataType::Integer(value) => value.to_string(),
            DataType::Text(value) => value.clone(),
            DataType::Boolean(value) => value.to_string(),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer(value) => write!(f, "{}", value),
            DataType::Text(value) => write!(f, "\"{}\"", value),
            DataType::Boolean(value) => write!(f, "{}", value),
        }
    }
}

impl FromStr for DataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_string().parse::<i32>() {
            Ok(value) => Ok(DataType::Integer(value as i64)),
            Err(_) => match s.to_string().parse::<bool>() {
                Ok(value) => Ok(DataType::Boolean(value)),
                Err(_) => Ok(DataType::Text(s.to_string())),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// Represents a row in a database table.
pub struct Row {
    /// The data in the row.
    pub data: HashMap<String, DataType>,
}
