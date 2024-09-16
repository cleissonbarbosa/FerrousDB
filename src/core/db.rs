use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
};

use druid::Data;
use serde::{Deserialize, Serialize};

use super::{row::Row, table::Table};
use crate::core::parser::command::SQLCommand;
use crate::core::parser::sql_parser::parse_sql;

pub enum PageResult<'a> {
    TableNotFound,
    PageOutOfRange,
    Page(Vec<&'a Row>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
/// Represents the FerrousDB database.
pub struct FerrousDB {
    pub tables: HashMap<String, Table>,
    is_loaded: bool,
}

impl Data for FerrousDB {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl FerrousDB {
    pub fn new() -> Self {
        FerrousDB {
            tables: HashMap::new(),
            is_loaded: false,
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<&str>) {
        let table = Table {
            name: name.to_string(),
            columns: columns.into_iter().map(|s| s.to_string()).collect(),
            rows: Vec::new(),
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

    pub fn delete_from(
        &mut self,
        table_name: &str,
        condition: Option<Box<dyn Fn(&Row) -> bool>>,
    ) -> Result<usize, String> {
        if let Some(table) = self.tables.get_mut(table_name) {
            let initial_count = table.rows.len();

            if let Some(cond) = condition {
                table.rows.retain(|row| !cond(row));
            } else {
                table.rows.clear();
            }

            let deleted_count = initial_count - table.rows.len();

            self.save_to_file("data.ferrous")
                .map_err(|e| format!("Failed to save to file: {}", e))?;

            Ok(deleted_count)
        } else {
            Err(format!("Table '{}' not found", table_name))
        }
    }

    pub fn get_page(
        &mut self,
        table_name: &str,
        page_number: usize,
        page_size: usize,
    ) -> PageResult {
        if !self.is_loaded {
            if let Ok(mut db) = FerrousDB::load_from_file("data.ferrous") {
                self.tables.extend(db.tables.drain());
                self.is_loaded = true;
            }
        }

        if let Some(table) = self.tables.iter().find(|t| t.1.name == table_name) {
            match table.1.get_page(page_number, page_size) {
                Some(page) => PageResult::Page(page),
                None => PageResult::PageOutOfRange,
            }
        } else {
            PageResult::TableNotFound
        }
    }

    pub fn total_pages(&self, table_name: &str, page_size: usize) -> Option<usize> {
        self.tables
            .iter()
            .find(|t| t.1.name == table_name)
            .map(|table| table.1.total_pages(page_size))
    }

    pub fn execute_sql(&mut self, sql: &str) -> Result<String, String> {
        let command = parse_sql(sql)?;
        match command {
            SQLCommand::CreateTable { name, columns } => {
                let columns_ref: Vec<&str> = columns.iter().map(AsRef::as_ref).collect();
                self.create_table(&name, columns_ref);
                Ok(format!("Table '{}' created successfully", name))
            }
            SQLCommand::InsertInto { table, values } => {
                self.insert_into(&table, values);
                Ok(format!("Data inserted into table '{}' successfully", table))
            }
            SQLCommand::SelectFrom {
                table,
                page_size,
                page,
            } => match self.get_page(&table, page, page_size) {
                PageResult::TableNotFound => Err(format!("Table '{}' not found", table)),
                PageResult::PageOutOfRange => Err(format!(
                    "Page number {} out of range for table '{}'",
                    page, table
                )),
                PageResult::Page(rows) => {
                    for row in rows {
                        println!("{:?}", row);
                    }
                    if let Some(total_pages) = self.total_pages(&table, page_size) {
                        println!("Page {} of {}", page, total_pages);
                    }
                    Ok(format!("Data selected from table '{}' successfully", table))
                }
            },
            SQLCommand::DeleteFrom { table, condition } => {
                let condition_fn = condition.map(|c| {
                    Box::new(move |row: &Row| {
                        // This is a simple implementation. You might want to expand this
                        // to handle more complex conditions.
                        row.data.iter().any(|(k, v)| format!("{}={}", k, v) == c)
                    }) as Box<dyn Fn(&Row) -> bool>
                });

                match self.delete_from(&table, condition_fn) {
                    Ok(count) => Ok(format!("{} row(s) deleted from table '{}'", count, table)),
                    Err(e) => Err(e),
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
        db.create_table("users", vec!["name", "age"]);
        assert_eq!(db.tables.len(), 1);
        assert_eq!(db.tables.get("users").unwrap().columns, vec!["name", "age"]);
    }

    #[test]
    fn test_insert_into() {
        let mut db = FerrousDB::new();
        db.create_table("users", vec!["name", "age"]);
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
    fn test_select_from_with_limit_and_offset() {
        let mut db = FerrousDB::new();
        db.create_table("users", vec!["name", "age"]);

        // Insert 5 users
        for i in 1..=5 {
            let mut values = HashMap::new();
            values.insert("name".to_string(), format!("User{}", i));
            values.insert("age".to_string(), format!("{}", 20 + i));
            db.insert_into("users", values);
        }

        // Test with limit 2 and offset 1
        let table = db.get_page("users", 2, 2);
        match table {
            PageResult::Page(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].data.get("name").unwrap(), "User3");
                assert_eq!(rows[1].data.get("name").unwrap(), "User4");
            }
            _ => {}
        };

        // Test with limit 3 and offset 3
        let table = db.get_page("users", 3, 2);
        match table {
            PageResult::Page(rows) => {
                assert_eq!(rows.len(), 1); // Only 2 rows left
                assert_eq!(rows[0].data.get("name").unwrap(), "User5");
            }
            _ => {}
        };
    }
}
