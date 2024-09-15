use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum SQLCommand {
    CreateTable {
        name: String,
        columns: Vec<String>,
        page_size: usize
    },
    InsertInto {
        table: String,
        values: HashMap<String, String>,
    },
    SelectFrom {
        table: String,
        page: usize
    },
}

impl SQLCommand {
    pub fn to_string(&self) -> String {
        match self {
            SQLCommand::CreateTable { name, columns, page_size } => {
                format!("CREATE TABLE {} ({}) PAGE SIZE {}", name, columns.join(", "), page_size)
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
            SQLCommand::SelectFrom { table, page } => format!("SELECT * FROM {} PAGE {}", table, page),
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
                let page_size = 100;
                SQLCommand::CreateTable { name, columns, page_size }
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
                let page = iter.next().unwrap().parse::<usize>().unwrap();
                SQLCommand::SelectFrom { table, page }
            }
            _ => panic!("Invalid command"),
        }
    }
}