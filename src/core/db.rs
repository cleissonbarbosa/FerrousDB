use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
};

use serde::{Deserialize, Serialize};

use super::{row::Row, table::Table};
use crate::core::parser::command::SQLCommand;
use crate::core::parser::sql_parser::parse_sql;

#[derive(Serialize, Deserialize)]
/// Represents the FerrousDB database.
pub struct FerrousDB {
    tables: HashMap<String, Table>,
    is_loaded: bool,
}

impl FerrousDB {
    pub fn new() -> Self {
        FerrousDB {
            tables: HashMap::new(),
            is_loaded: false,
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<&str>, page_size: usize) {
        let table = Table {
            name: name.to_string(),
            columns: columns.into_iter().map(|s| s.to_string()).collect(),
            rows: Vec::new(),
            page_size,
        };
        self.tables.insert(name.to_string(), table);
        self.save_to_file("data.ferrous")
            .expect("Failed to save to file");
    }

    pub fn insert_into(&mut self, table_name: &str, values: HashMap<String, String>) {
        if let Some(table) = self.tables.get_mut(table_name) {
            let row = Row { data: values };
            table.rows.push(row);
            self.save_to_file("data.ferrous")
                .expect("Failed to save to file");
        }
    }

    pub fn select_from(&mut self, table_name: &str) -> Option<&Table> {
        if !self.is_loaded {
            if let Ok(mut db) = FerrousDB::load_from_file("data.ferrous") {
                self.tables.extend(db.tables.drain());
                self.is_loaded = true;
            }
        }
        self.tables.get(table_name)
    }

    pub fn get_page(&self, table_name: &str, page_number: usize) -> Option<Vec<&Row>> {
        self.tables
            .iter()
            .find(|t| t.1.name == table_name)
            .map(|table| table.1.get_page(page_number))
    }

    pub fn total_pages(&self, table_name: &str) -> Option<usize> {
        self.tables
            .iter()
            .find(|t| t.1.name == table_name)
            .map(|table| table.1.total_pages())
    }

    pub fn execute_sql(&mut self, sql: &str) -> Result<String, String> {
        let command = parse_sql(sql)?;
        match command {
            SQLCommand::CreateTable {
                name,
                columns,
                page_size,
            } => {
                let columns_ref: Vec<&str> = columns.iter().map(AsRef::as_ref).collect();
                self.create_table(&name, columns_ref, page_size);
                Ok(format!("Table '{}' created successfully", name))
            }
            SQLCommand::InsertInto { table, values } => {
                self.insert_into(&table, values);
                Ok(format!("Data inserted into table '{}' successfully", table))
            }
            SQLCommand::SelectFrom { table, page } => {
                if let Some(rows) = self.get_page(&table, page) {
                    for row in rows {
                        println!("{:?}", row);
                    }
                    if let Some(total_pages) = self.total_pages(&table) {
                        println!("Page {} of {}", page, total_pages);
                    }
                    Ok(format!("Data selected from table '{}' successfully", table))
                } else {
                    Err(format!("Table '{}' not found", table))
                }
            }
        }
    }

    fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let encoded: Vec<u8> = bincode::serialize(&self).expect("Failed to serialize database");
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn load_from_file(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let db: FerrousDB = bincode::deserialize(&buffer).expect("Failed to deserialize database");
        Ok(db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table() {
        let mut db = FerrousDB::new();
        db.create_table("users", vec!["name", "age"], 100);
        assert_eq!(db.tables.len(), 1);
        assert_eq!(db.tables.get("users").unwrap().columns, vec!["name", "age"]);
    }

    #[test]
    fn test_insert_into() {
        let mut db = FerrousDB::new();
        db.create_table("users", vec!["name", "age"], 100);
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());
        values.insert("age".to_string(), "30".to_string());
        db.insert_into("users", values);
        assert_eq!(db.tables.get("users").unwrap().rows.len(), 1);
        let row = &db.tables.get("users").unwrap().rows[0];
        assert_eq!(row.data.get("name").unwrap(), "Alice");
        assert_eq!(row.data.get("age").unwrap(), "30");
    }

    #[test]
    fn test_select_from() {
        let mut db = FerrousDB::new();
        db.create_table("users", vec!["name", "age"], 100);
        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());
        values.insert("age".to_string(), "30".to_string());
        db.insert_into("users", values);
        let table = db.select_from("users").unwrap();
        assert_eq!(table.columns, vec!["name", "age"]);
        assert_eq!(table.rows.len(), 1);
        let row = &table.rows[0];
        assert_eq!(row.data.get("name").unwrap(), "Alice");
        assert_eq!(row.data.get("age").unwrap(), "30");
    }
}
