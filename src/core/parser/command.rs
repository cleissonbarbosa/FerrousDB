use std::collections::HashMap;

use crate::{core::table::ColumnSchema, DataType};

#[derive(Debug, PartialEq)]
pub enum SQLCommand {
    CreateTable {
        name: String,
        columns: Vec<ColumnSchema>,
    },
    InsertInto {
        table: String,
        values: HashMap<String, DataType>,
    },
    SelectFrom {
        table: String,
        page_size: usize,
        page: usize,
        group_by: Option<String>,
        order_by: Option<(String, bool)>, // (column_name, is_ascending)
    },
    DeleteFrom {
        table: String,
        condition: Option<String>,
    },
    Update {
        table: String,
        assignments: HashMap<String, DataType>,
        condition: Option<String>,
    },
}

impl SQLCommand {
    pub fn to_string(&self) -> String {
        match self {
            SQLCommand::CreateTable { name, columns } => {
                format!(
                    "CREATE TABLE {} ({})",
                    name,
                    columns
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            SQLCommand::InsertInto { table, values } => {
                let columns: Vec<String> = values.keys().map(|s| s.to_string()).collect();
                let values: Vec<String> = values.values().map(|s| s.to_string()).collect();
                format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    table,
                    columns.join(", "),
                    values.join(", ")
                )
            }
            SQLCommand::SelectFrom {
                table,
                page_size,
                page,
                group_by,
                order_by,
            } => {
                let mut query = format!(
                    "SELECT * FROM {} LIMIT {} OFFSET {}",
                    table, page_size, page
                );
                if let Some(group_by) = group_by {
                    query.push_str(&format!(" GROUP BY {}", group_by));
                }
                if let Some((column, is_ascending)) = order_by {
                    query.push_str(&format!(
                        " ORDER BY {} {}",
                        column,
                        if *is_ascending { "ASC" } else { "DESC" }
                    ));
                }
                query
            }
            SQLCommand::DeleteFrom { table, condition } => {
                let condition_str = condition
                    .as_ref()
                    .map(|c| format!(" WHERE {}", c))
                    .unwrap_or_else(|| String::new());
                format!("DELETE FROM {}{}", table, condition_str)
            }
            SQLCommand::Update {
                table,
                assignments,
                condition,
            } => {
                let assignments_str = assignments
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ");
                let condition_str = condition
                    .as_ref()
                    .map(|c| format!(" WHERE {}", c))
                    .unwrap_or_else(|| String::new());
                format!("UPDATE {} SET {}{}", table, assignments_str, condition_str)
            }
        }
    }
}

impl FromIterator<String> for SQLCommand {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let command = iter.next().unwrap();
        match command.as_str() {
            "CREATE TABLE" => {
                let name = iter.next().unwrap();
                let columns = iter
                    .map(|s| s.parse::<ColumnSchema>().unwrap()) // Convert String to ColumnSchema
                    .collect::<Vec<ColumnSchema>>();
                SQLCommand::CreateTable { name, columns }
            }
            "INSERT INTO" => {
                let table = iter.next().unwrap();
                let values: HashMap<String, DataType> = iter
                    .map(|s| {
                        let mut parts = s.splitn(2, '=');
                        let key = parts.next().unwrap().to_string();
                        let value = parts.next().unwrap().to_string();
                        (key, value.parse::<DataType>().unwrap())
                    })
                    .collect();
                SQLCommand::InsertInto { table, values }
            }
            "SELECT * FROM" => {
                let table = iter.next().unwrap();
                let page_size = iter
                    .next()
                    .unwrap_or_else(|| "1000".to_string())
                    .parse::<usize>()
                    .unwrap();
                let page = iter
                    .next()
                    .unwrap_or_else(|| "1".to_string())
                    .parse::<usize>()
                    .unwrap();
                let group_by = iter.next();
                let order_by = iter.next().map(|s| {
                    let mut parts = s.splitn(2, ' ');
                    let column = parts.next().unwrap().to_string();
                    let is_ascending = parts.next().unwrap_or("ASC") == "ASC";
                    (column, is_ascending)
                });
                SQLCommand::SelectFrom {
                    table,
                    page_size,
                    page,
                    group_by,
                    order_by,
                }
            }
            "DELETE FROM" => {
                let table = iter.next().unwrap();
                let condition = iter.next().map(|s| s.to_string());
                SQLCommand::DeleteFrom { table, condition }
            }
            "UPDATE" => {
                let table = iter.next().unwrap();
                let assignments: HashMap<String, DataType> = iter
                    .map(|s| {
                        let mut parts = s.splitn(2, '=');
                        let key = parts.next().unwrap().to_string();
                        let value = parts.next().unwrap().to_string();
                        (key, value.parse::<DataType>().unwrap())
                    })
                    .collect();
                let condition = iter.next().map(|s| s.to_string());
                SQLCommand::Update {
                    table,
                    assignments,
                    condition,
                }
            }
            _ => panic!("Invalid command"),
        }
    }
}
