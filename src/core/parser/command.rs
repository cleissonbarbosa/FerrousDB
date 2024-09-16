use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum SQLCommand {
    CreateTable {
        name: String,
        columns: Vec<String>
    },
    InsertInto {
        table: String,
        values: HashMap<String, String>,
    },
    SelectFrom {
        table: String,
        page_size: usize,
        page: usize,
    },
    DeleteFrom {
        table: String,
        condition: Option<String>,
    }
}

impl SQLCommand {
    pub fn to_string(&self) -> String {
        match self {
            SQLCommand::CreateTable { name, columns } => {
                format!("CREATE TABLE {} ({})", name, columns.join(", "))
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
            SQLCommand::SelectFrom { table, page_size, page, } => format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, page_size, page),
            SQLCommand::DeleteFrom { table, condition } => {
                let condition_str = condition
                    .as_ref()
                    .map(|c| format!(" WHERE {}", c))
                    .unwrap_or_else(|| String::new());
                format!("DELETE FROM {}{}", table, condition_str)
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
                let columns = iter.collect();
                SQLCommand::CreateTable { name, columns }
            }
            "INSERT INTO" => {
                let table = iter.next().unwrap();
                let values: HashMap<String, String> = iter
                    .map(|s| {
                        let mut parts = s.splitn(2, '=');
                        let key = parts.next().unwrap().to_string();
                        let value = parts.next().unwrap().to_string();
                        (key, value)
                    })
                    .collect();
                SQLCommand::InsertInto { table, values }
            }
            "SELECT * FROM" => {
                let table = iter.next().unwrap();
                let page_size = iter.next().unwrap_or_else(|| "1000".to_string()).parse::<usize>().unwrap();
                let page = iter.next().unwrap_or_else(|| "1".to_string()).parse::<usize>().unwrap();
                SQLCommand::SelectFrom { table, page_size, page }
            }
            "DELETE FROM" => {
                let table = iter.next().unwrap();
                let condition = iter.next().map(|s| s.to_string());
                SQLCommand::DeleteFrom { table, condition }
            }
            _ => panic!("Invalid command"),
        }
    }
}