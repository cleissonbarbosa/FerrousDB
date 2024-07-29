use serde::{Serialize, Deserialize};

use super::row::Row;

#[derive(Serialize, Deserialize, Debug)]
/// Represents a table in the database.
pub struct Table {
    /// The name of the table.
    pub name: String,
    /// The columns of the table.
    pub columns: Vec<String>,
    /// The rows of the table.
    pub rows: Vec<Row>,
}