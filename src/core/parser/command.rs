use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum SQLCommand {
    CreateTable {
        name: String,
        columns: Vec<String>,
    },
    InsertInto {
        table: String,
        values: HashMap<String, String>,
    },
    SelectFrom {
        table: String,
    },
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
            SQLCommand::SelectFrom { table } => format!("SELECT * FROM {}", table),
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
                SQLCommand::SelectFrom { table }
            }
            _ => panic!("Invalid command"),
        }
    }
}