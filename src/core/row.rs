use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// Represents a row in a database table.
pub struct Row {
    /// The data in the row.
    pub data: HashMap<String, String>,
}