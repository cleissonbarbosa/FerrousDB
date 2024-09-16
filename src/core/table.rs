use druid::Data;
use serde::{Deserialize, Serialize};

use super::row::Row;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// Represents a table in the database.
pub struct Table {
    /// The name of the table.
    pub name: String,
    /// The columns of the table.
    pub columns: Vec<String>,
    /// The rows of the table.
    pub rows: Vec<Row>,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>) -> Self {
        Table {
            name,
            columns,
            rows: Vec::new(),
        }
    }

    pub fn get_page(&self, mut page_number: usize, page_size: usize) -> Option<Vec<&Row>> {
        if page_number > self.total_pages(page_size) {
            return None;
        }

        if page_number < 0 {
            return Some(self.rows.iter().collect());
        }

        if page_number == 0 {
            println!("WARN: Page start with 1 not 0");
            page_number = 1;
        }

        let start = (page_number - 1) * page_size;
        let end = start + page_size;
        Some(self.rows[start..end.min(self.rows.len())].iter().collect())
    }

    pub fn total_pages(&self, page_size: usize) -> usize {
        (self.rows.len() + page_size - 1) / page_size
    }
}
