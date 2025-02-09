use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
};

use druid::Data;
use serde::{Deserialize, Serialize};

use super::{
    error_handling::FerrousDBError,
    index::{Index, IndexType},
    row::Row,
    table::{ColumnSchema, Constraint, Table},
    write_ahead_log::WriteAheadLog,
};
use crate::{core::parser::command::SQLCommand, core::parser::sql_parser::parse_sql, DataType};

pub enum PageResult<'a> {
    TableNotFound,
    PageOutOfRange,
    Page(Vec<&'a Row>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
/// Represents the FerrousDB database.
pub struct FerrousDB {
    pub tables: HashMap<String, Table>,
    pub indexes: HashMap<String, Index>,
    is_loaded: bool,
}

impl Data for FerrousDB {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

impl Default for FerrousDB {
    fn default() -> Self {
        Self::new()
    }
}

impl FerrousDB {
    pub fn new() -> Self {
        match FerrousDB::load_from_file("data.ferrous") {
            Ok(db) => db,
            Err(_) => FerrousDB {
                tables: HashMap::new(),
                indexes: HashMap::new(),
                is_loaded: false,
            },
        }
    }

    pub fn create_table(
        &mut self,
        name: &str,
        columns: Vec<ColumnSchema>,
    ) -> Result<(), FerrousDBError> {
        if self.tables.contains_key(name) {
            return Err(FerrousDBError::TableExists(name.to_string()));
        }

        let table = Table {
            name: name.to_string(),
            schema: columns,
            rows: Vec::new(),
        };
        self.tables.insert(name.to_string(), table);
        self.save_to_file("data.ferrous")
            .expect("Failed to save to file");
        Ok(())
    }

    pub fn create_index(
        &mut self,
        table_name: &str,
        column_name: &str,
        index_type: IndexType,
    ) -> Result<(), FerrousDBError> {
        let table = self
            .tables
            .get(table_name)
            .ok_or_else(|| FerrousDBError::TableNotFound(table_name.to_string()))?;

        // Verify column exists
        if !table.schema.iter().any(|col| col.name == column_name) {
            return Err(FerrousDBError::ColumnNotFound(column_name.to_string()));
        }

        let index_name = format!("{}_{}", table_name, column_name);
        let mut index = Index::new(table_name.to_string(), column_name.to_string(), index_type);

        // Build initial index
        for (row_idx, row) in table.rows.iter().enumerate() {
            if let Some(value) = row.data.get(column_name) {
                index.insert(value.clone(), row_idx);
            }
        }

        self.indexes.insert(index_name, index);
        self.save_to_file("data.ferrous")?;
        Ok(())
    }

    fn validate_constraints(
        &self,
        table_name: &str,
        values: &HashMap<String, DataType>,
    ) -> Result<(), FerrousDBError> {
        let table = self
            .tables
            .get(table_name)
            .ok_or_else(|| FerrousDBError::TableNotFound(table_name.to_string()))?;

        for column in &table.schema {
            for constraint in &column.constraints {
                match constraint {
                    Constraint::NotNull => {
                        if !values.contains_key(&column.name) {
                            return Err(FerrousDBError::ConstraintViolation(format!(
                                "NOT NULL constraint failed: {}",
                                column.name
                            )));
                        }
                    }
                    Constraint::Unique => {
                        if let Some(value) = values.get(&column.name) {
                            for row in &table.rows {
                                if let Some(existing_value) = row.data.get(&column.name) {
                                    if existing_value == value {
                                        return Err(FerrousDBError::ConstraintViolation(format!(
                                            "UNIQUE constraint failed: {}",
                                            column.name
                                        )));
                                    }
                                }
                            }
                        }
                    }
                    Constraint::ForeignKey {
                        ref_table,
                        ref_column,
                    } => {
                        if let Some(value) = values.get(&column.name) {
                            let referenced_table = self
                                .tables
                                .get(ref_table)
                                .ok_or_else(|| FerrousDBError::TableNotFound(ref_table.clone()))?;

                            let found = referenced_table.rows.iter().any(|row| {
                                if let Some(ref_value) = row.data.get(ref_column) {
                                    ref_value == value
                                } else {
                                    false
                                }
                            });

                            if !found {
                                return Err(FerrousDBError::ConstraintViolation(format!(
                                    "FOREIGN KEY constraint failed: {} references {}.{}",
                                    column.name, ref_table, ref_column
                                )));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn insert_into(
        &mut self,
        table_name: &str,
        values: HashMap<String, DataType>,
    ) -> Result<(), FerrousDBError> {
        // Validate constraints before inserting
        self.validate_constraints(table_name, &values)?;

        if let Some(table) = self.tables.get_mut(table_name) {
            // Check data types match the schema
            for (column_name, value) in &values {
                let column_schema = table
                    .schema
                    .iter()
                    .find(|col| &col.name == column_name)
                    .ok_or(FerrousDBError::ColumnNotFound(column_name.clone()))?;
                if value.get_type() != column_schema.data_type {
                    return Err(FerrousDBError::TypeMismatch(column_name.clone()));
                }
            }
            let row_index = table.rows.len();
            let row = Row {
                data: values.clone(),
            };
            table.rows.push(row);

            // Update indexes
            for (column_name, value) in values {
                let index_name = format!("{}_{}", table_name, column_name);
                if let Some(index) = self.indexes.get_mut(&index_name) {
                    index.insert(value, row_index);
                }
            }

            self.save_to_file("data.ferrous")?;
            Ok(())
        } else {
            Err(FerrousDBError::TableNotFound(table_name.to_string()))
        }
    }

    pub fn update(
        &mut self,
        table_name: &str,
        assignments: HashMap<String, DataType>,
        condition: Option<String>,
    ) -> Result<usize, FerrousDBError> {
        // Validate constraints before updating
        self.validate_constraints(table_name, &assignments)?;

        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| FerrousDBError::TableNotFound(table_name.to_string()))?;

        let mut updated_count = 0;
        for (row_idx, row) in table.rows.iter_mut().enumerate() {
            let should_update = match &condition {
                Some(cond) => {
                    let parts: Vec<&str> = cond.split('=').collect();
                    if parts.len() != 2 {
                        return Err(FerrousDBError::ParseError(
                            "Invalid condition format".to_string(),
                        ));
                    }
                    if let Some(value) = row.data.get(parts[0].trim()) {
                        match value {
                            DataType::Integer(n) => {
                                if let Ok(comparison_value) = parts[1].trim().trim_matches('\'').parse::<i64>() {
                                    *n == comparison_value
                                } else {
                                    false
                                }
                            },
                            DataType::Text(s) => s == parts[1].trim().trim_matches('\''),
                            DataType::Boolean(b) => {
                                if let Ok(comparison_value) = parts[1].trim().trim_matches('\'').parse::<bool>() {
                                    *b == comparison_value
                                } else {
                                    false
                                }
                            }
                        }
                    } else {
                        false
                    }
                }
                None => true,
            };

            if should_update {
                // Update indexes before modifying the row
                for (col, new_value) in &assignments {
                    let index_name = format!("{}_{}", table_name, col);
                    if let Some(index) = self.indexes.get_mut(&index_name) {
                        if let Some(old_value) = row.data.get(col) {
                            index.update(old_value, new_value.clone(), row_idx);
                        }
                    }
                }

                // Update the row
                for (col, value) in &assignments {
                    if !table.schema.iter().any(|c| &c.name == col) {
                        return Err(FerrousDBError::ColumnNotFound(col.clone()));
                    }
                    row.data.insert(col.clone(), value.clone());
                }
                updated_count += 1;
            }
        }

        self.save_to_file("data.ferrous")?;
        Ok(updated_count)
    }

    pub fn delete_from(
        &mut self,
        table_name: &str,
        condition: Option<String>,
    ) -> Result<usize, FerrousDBError> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| FerrousDBError::TableNotFound(table_name.to_string()))?;

        let initial_count = table.rows.len();
        let mut rows_to_delete = Vec::new();

        // First pass: identify rows to delete
        if let Some(cond) = condition {
            let parts: Vec<&str> = cond.split('=').collect();
            if parts.len() != 2 {
                return Err(FerrousDBError::ParseError(
                    "Invalid condition format".to_string(),
                ));
            }
            let column = parts[0].trim();
            let value = parts[1].trim().trim_matches('\'');

            for (idx, row) in table.rows.iter().enumerate() {
                if let Some(row_value) = row.data.get(column) {
                    let should_delete = match row_value {
                        DataType::Integer(n) => {
                            if let Ok(comparison_value) = value.parse::<i64>() {
                                *n == comparison_value
                            } else {
                                false
                            }
                        },
                        DataType::Text(s) => s == value,
                        DataType::Boolean(b) => {
                            if let Ok(comparison_value) = value.parse::<bool>() {
                                *b == comparison_value
                            } else {
                                false
                            }
                        }
                    };
                    if should_delete {
                        rows_to_delete.push(idx);
                    }
                }
            }
        } else {
            rows_to_delete.extend(0..table.rows.len());
        }

        // Update indexes
        for idx in &rows_to_delete {
            let row = &table.rows[*idx];
            for (col_name, value) in &row.data {
                let index_name = format!("{}_{}", table_name, col_name);
                if let Some(index) = self.indexes.get_mut(&index_name) {
                    index.remove(value, *idx);
                }
            }
        }

        // Remove rows in reverse order to maintain correct indices
        rows_to_delete.sort_unstable_by(|a, b| b.cmp(a));
        for idx in rows_to_delete {
            table.rows.remove(idx);
        }

        let deleted_count = initial_count - table.rows.len();
        self.save_to_file("data.ferrous")?;
        Ok(deleted_count)
    }

    pub fn get_page(
        &mut self,
        table_name: &str,
        page_number: usize,
        page_size: usize,
        group_by: Option<String>,
        order_by: Option<(String, bool)>,
    ) -> PageResult {
        if !self.is_loaded {
            if let Ok(mut db) = FerrousDB::load_from_file("data.ferrous") {
                self.tables.extend(db.tables.drain());
                self.is_loaded = true;
            }
        }

        if let Some(table) = self.tables.get(table_name) {
            let mut rows: Vec<&Row> = table.rows.iter().collect();

            // Apply GROUP BY if specified
            if let Some(group_by_col) = group_by {
                let mut grouped_rows = HashMap::new();
                for row in rows {
                    if let Some(value) = row.data.get(&group_by_col) {
                        grouped_rows
                            .entry(value.clone())
                            .or_insert_with(Vec::new)
                            .push(row);
                    }
                }
                rows = grouped_rows
                    .into_iter()
                    .map(|(_, group)| group[0])
                    .collect();
            }

            // Apply ORDER BY if specified
            if let Some((col, is_ascending)) = order_by {
                rows.sort_by(|a, b| {
                    let a_val = a.data.get(&col).map(|v| v.get_value());
                    let b_val = b.data.get(&col).map(|v| v.get_value());
                    if is_ascending {
                        a_val.cmp(&b_val)
                    } else {
                        b_val.cmp(&a_val)
                    }
                });
            }

            let start = (page_number - 1) * page_size;
            let end = start + page_size;

            if start >= rows.len() {
                PageResult::PageOutOfRange
            } else {
                PageResult::Page(rows[start..end.min(rows.len())].to_vec())
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

    pub fn execute_sql(&mut self, sql: &str) -> Result<String, FerrousDBError> {
        let mut wal = WriteAheadLog::new("ferrousdb.log").unwrap();
        wal.log(sql).unwrap();
        let command = parse_sql(sql)?;
        match command {
            SQLCommand::CreateTable { name, columns } => {
                let columns_ref: Vec<ColumnSchema> = columns;
                self.create_table(&name, columns_ref).unwrap();
                Ok(format!("Table '{}' created successfully", name))
            }
            SQLCommand::InsertInto { table, values } => {
                self.insert_into(&table, values).unwrap();
                Ok(format!("Data inserted into table '{}' successfully", table))
            }
            SQLCommand::SelectFrom {
                table,
                page_size,
                page,
                group_by,
                order_by,
            } => match self.get_page(&table, page, page_size, group_by, order_by) {
                PageResult::TableNotFound => Err(FerrousDBError::TableNotFound(format!(
                    "Table '{}' not found",
                    table
                ))),
                PageResult::PageOutOfRange => Err(FerrousDBError::ParseError(format!(
                    "Page number {} out of range for table '{}'",
                    page, table
                ))),
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
                match self.delete_from(&table, condition) {
                    Ok(count) => Ok(format!("{} row(s) deleted from table '{}'", count, table)),
                    Err(e) => Err(e),
                }
            }
            SQLCommand::Update {
                table,
                assignments,
                condition,
            } => match self.update(&table, assignments, condition) {
                Ok(count) => Ok(format!("{} row(s) updated in table '{}'", count, table)),
                Err(e) => Err(e),
            },
            SQLCommand::CreateView {
                name,
                query,
                columns,
            } => {
                // Por enquanto apenas retornamos um erro indicando que a feature não está implementada
                Err(FerrousDBError::ParseError(
                    "Views are not implemented yet".to_string(),
                ))
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
    use std::fs;

    fn setup() -> FerrousDB {
        // Remove any existing test database file
        let _ = fs::remove_file("data.ferrous");
        FerrousDB::new()
    }

    #[test]
    fn test_create_table() {
        let mut db = setup();
        let result = db.create_table(
            "users",
            vec![
                ColumnSchema::new("name".to_string(), "TEXT".to_string()),
                ColumnSchema::new("age".to_string(), "INTEGER".to_string()),
            ],
        );
        assert!(result.is_ok());
        assert_eq!(db.tables.len(), 1);
        assert_eq!(
            db.tables.get("users").unwrap().schema,
            vec![
                ColumnSchema::new("name".to_string(), "TEXT".to_string()),
                ColumnSchema::new("age".to_string(), "INTEGER".to_string()),
            ]
        );
    }

    #[test]
    fn test_insert_into() {
        let mut db = setup();
        let create_result = db.create_table(
            "users",
            vec![
                ColumnSchema::new("name".to_string(), "TEXT".to_string()),
                ColumnSchema::new("age".to_string(), "INTEGER".to_string()),
            ],
        );
        assert!(create_result.is_ok());

        let mut values = HashMap::new();
        values.insert("name".to_string(), DataType::Text("Alice".to_string()));
        values.insert("age".to_string(), DataType::Integer(30));
        let insert_result = db.insert_into("users", values);
        assert!(insert_result.is_ok());

        assert_eq!(db.tables.get("users").unwrap().rows.len(), 1);
        let row = &db.tables.get("users").unwrap().rows[0];
        assert_eq!(row.data.get("name").unwrap().get_value(), "Alice");
        assert_eq!(row.data.get("age").unwrap().get_value(), "30");
    }

    #[test]
    fn test_select_from_with_limit_and_offset() {
        let mut db = setup();
        let create_result = db.create_table(
            "users",
            vec![
                ColumnSchema::new("name".to_string(), "TEXT".to_string()),
                ColumnSchema::new("age".to_string(), "INTEGER".to_string()),
            ],
        );
        assert!(create_result.is_ok());

        // Insert 5 users
        for i in 1..=5 {
            let mut values = HashMap::new();
            values.insert("name".to_string(), DataType::Text(format!("User{}", i)));
            values.insert("age".to_string(), DataType::Integer(20 + i));
            let insert_result = db.insert_into("users", values);
            assert!(insert_result.is_ok());
        }

        // Test with limit 2 and offset 1
        let table = db.get_page("users", 2, 2, None, None);
        match table {
            PageResult::Page(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].data.get("name").unwrap().get_value(), "User3");
                assert_eq!(rows[1].data.get("name").unwrap().get_value(), "User4");
            }
            _ => panic!("Expected PageResult::Page"),
        };

        // Test with limit 3 and offset 3
        let table = db.get_page("users", 3, 2, None, None);
        match table {
            PageResult::Page(rows) => {
                assert_eq!(rows.len(), 1); // Only 1 row left
                assert_eq!(rows[0].data.get("name").unwrap().get_value(), "User5");
            }
            _ => panic!("Expected PageResult::Page"),
        };
    }

    #[test]
    fn test_update() {
        let mut db = setup();
        let create_result = db.create_table(
            "users",
            vec![
                ColumnSchema::new("name".to_string(), "TEXT".to_string()),
                ColumnSchema::new("age".to_string(), "INTEGER".to_string()),
            ],
        );
        assert!(create_result.is_ok());

        // Insert initial data
        let mut values = HashMap::new();
        values.insert("name".to_string(), DataType::Text("Alice".to_string()));
        values.insert("age".to_string(), DataType::Integer(30));
        let insert_result = db.insert_into("users", values);
        assert!(insert_result.is_ok());

        // Verify initial data
        match db.get_page("users", 1, 10, None, None) {
            PageResult::Page(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].data.get("name").unwrap().get_value(), "Alice");
                assert_eq!(rows[0].data.get("age").unwrap().get_value(), "30");
            }
            _ => panic!("Expected to find inserted row"),
        }

        // Perform update
        let mut assignments = HashMap::new();
        assignments.insert("age".to_string(), DataType::Integer(31));
        let update_result = db.update("users", assignments, Some("name = 'Alice'".to_string()));
        assert!(update_result.is_ok());
        assert_eq!(update_result.unwrap(), 1); // Should update exactly 1 row

        // Verify the update was successful
        match db.get_page("users", 1, 10, None, None) {
            PageResult::Page(rows) => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].data.get("name").unwrap().get_value(), "Alice");
                assert_eq!(rows[0].data.get("age").unwrap().get_value(), "31");
            }
            _ => panic!("Expected to find the updated row"),
        }
    }

    #[test]
    fn test_delete_from() {
        let mut db = setup();
        let create_result = db.create_table(
            "users",
            vec![
                ColumnSchema::new("name".to_string(), "TEXT".to_string()),
                ColumnSchema::new("age".to_string(), "INTEGER".to_string()),
            ],
        );
        assert!(create_result.is_ok());

        let mut values = HashMap::new();
        values.insert("name".to_string(), DataType::Text("Alice".to_string()));
        values.insert("age".to_string(), DataType::Integer(30));
        let insert_result = db.insert_into("users", values);
        assert!(insert_result.is_ok());

        let delete_result = db.delete_from("users", Some("name = 'Alice'".to_string()));
        assert!(delete_result.is_ok());

        assert_eq!(db.tables.get("users").unwrap().rows.len(), 0);
    }
}
