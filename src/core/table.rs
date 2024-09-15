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
    /// The page size for pagination.
    pub page_size: usize,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>, page_size: usize) -> Self {
        Table {
            name,
            columns,
            rows: Vec::new(),
            page_size,
        }
    }

    pub fn get_page(&self, page_number: usize) -> Vec<&Row> {
        let start = (page_number - 1) * self.page_size;
        let end = start + self.page_size;
        self.rows[start..end.min(self.rows.len())].iter().collect()
    }

    pub fn total_pages(&self) -> usize {
        (self.rows.len() + self.page_size - 1) / self.page_size
    }
}