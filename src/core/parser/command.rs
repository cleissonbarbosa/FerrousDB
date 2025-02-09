use crate::core::error_handling::FerrousDBError;
use crate::{core::table::ColumnSchema, DataType};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum SQLCommand {
    CreateTable {
        name: String,
        columns: Vec<ColumnSchema>,
    },
    CreateView {
        name: String,
        query: String,
        columns: Vec<String>,
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
            SQLCommand::CreateView {
                name,
                query,
                columns,
            } => {
                format!("CREATE VIEW {} ({}) AS {}", name, columns.join(", "), query)
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
            "CREATE VIEW" => {
                let name = iter.next().unwrap();
                let query = iter.next().unwrap();
                let columns = iter.collect::<Vec<String>>();
                SQLCommand::CreateView {
                    name,
                    query,
                    columns,
                }
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
                // Skip "SET"
                iter.next();
                // Collect all remaining items for processing
                let remaining: Vec<String> = iter.collect();
                let where_pos = remaining
                    .iter()
                    .position(|x| x.eq_ignore_ascii_case("WHERE"));

                let assignments: HashMap<String, DataType> = if let Some(pos) = where_pos {
                    let (assignments, where_clause) = remaining.split_at(pos);
                    let condition = where_clause[1..]
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join(" ");
                    assignments
                        .iter()
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            let mut parts = s.splitn(2, '=');
                            let key = parts.next().unwrap().to_string();
                            let value = parts.next().unwrap().to_string();
                            (key, value.parse::<DataType>().unwrap())
                        })
                        .collect()
                } else {
                    remaining
                        .iter()
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            let mut parts = s.splitn(2, '=');
                            let key = parts.next().unwrap().to_string();
                            let value = parts.next().unwrap().to_string();
                            (key, value.parse::<DataType>().unwrap())
                        })
                        .collect()
                };

                let condition = where_pos.map(|pos| remaining[pos + 1..].join(" "));

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

impl FromStr for SQLCommand {
    type Err = FerrousDBError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts: Vec<String> = s.split_whitespace().map(String::from).collect();

        if parts.is_empty() {
            return Err(FerrousDBError::ParseError("Empty command".to_string()));
        }

        let command = parts.remove(0).to_uppercase();
        match command.as_str() {
            "SELECT" => {
                if parts.is_empty() {
                    return Err(FerrousDBError::ParseError(
                        "Table name not provided".to_string(),
                    ));
                }
                let table = parts.remove(0);

                Ok(SQLCommand::SelectFrom {
                    table,
                    page_size: 1000,
                    page: 1,
                    group_by: None,
                    order_by: None,
                })
            }
            "INSERT" => {
                // Skip "INTO"
                if parts.len() < 2 || parts[0].to_uppercase() != "INTO" {
                    return Err(FerrousDBError::ParseError(
                        "Invalid INSERT syntax".to_string(),
                    ));
                }
                parts.remove(0); // remove "INTO"
                let table = parts.remove(0);

                let mut values = HashMap::new();
                for part in parts {
                    let mut pair = part.splitn(2, '=');
                    let key = pair
                        .next()
                        .ok_or_else(|| {
                            FerrousDBError::ParseError("Invalid key-value pair".to_string())
                        })?
                        .to_string();
                    let value = pair
                        .next()
                        .ok_or_else(|| {
                            FerrousDBError::ParseError("Invalid key-value pair".to_string())
                        })?
                        .parse::<DataType>()
                        .map_err(|_| {
                            FerrousDBError::ParseError("Invalid value type".to_string())
                        })?;
                    values.insert(key, value);
                }

                Ok(SQLCommand::InsertInto { table, values })
            }
            "UPDATE" => {
                if parts.len() < 4 || parts[1].to_uppercase() != "SET" {
                    return Err(FerrousDBError::ParseError(
                        "Invalid UPDATE syntax".to_string(),
                    ));
                }
                let table = parts.remove(0);
                parts.remove(0); // remove "SET"

                let where_pos = parts.iter().position(|x| x.to_uppercase() == "WHERE");

                let (assignments_part, condition) = if let Some(pos) = where_pos {
                    let (left, right) = parts.split_at(pos);
                    (left.to_vec(), Some(right[1..].join(" ")))
                } else {
                    (parts, None)
                };

                let mut assignments = HashMap::new();
                for part in assignments_part {
                    let mut pair = part.splitn(2, '=');
                    let key = pair
                        .next()
                        .ok_or_else(|| {
                            FerrousDBError::ParseError("Invalid assignment".to_string())
                        })?
                        .to_string();
                    let value = pair
                        .next()
                        .ok_or_else(|| {
                            FerrousDBError::ParseError("Invalid assignment".to_string())
                        })?
                        .parse::<DataType>()
                        .map_err(|_| {
                            FerrousDBError::ParseError("Invalid value type".to_string())
                        })?;
                    assignments.insert(key, value);
                }

                Ok(SQLCommand::Update {
                    table,
                    assignments,
                    condition,
                })
            }
            "DELETE" => {
                if parts.is_empty() || parts[0].to_uppercase() != "FROM" {
                    return Err(FerrousDBError::ParseError(
                        "Invalid DELETE syntax".to_string(),
                    ));
                }
                parts.remove(0); // remove "FROM"
                if parts.is_empty() {
                    return Err(FerrousDBError::ParseError(
                        "Table name not provided".to_string(),
                    ));
                }
                let table = parts.remove(0);

                let where_pos = parts.iter().position(|x| x.to_uppercase() == "WHERE");

                let condition = where_pos.map(|pos| parts[pos + 1..].join(" "));

                Ok(SQLCommand::DeleteFrom { table, condition })
            }
            _ => Err(FerrousDBError::ParseError(
                "Unsupported command".to_string(),
            )),
        }
    }
}
