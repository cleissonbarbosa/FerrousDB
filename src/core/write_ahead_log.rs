use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    core::parser::{command::SQLCommand, sql_parser::parse_sql},
    DataType,
};

pub struct WriteAheadLog {
    writer: BufWriter<File>,
    log_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogEntry {
    Command(String),
    SelectFrom {
        table: String,
        page_size: usize,
        page: usize,
        group_by: Option<String>,
        order_by: Option<(String, bool)>,
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

impl WriteAheadLog {
    pub fn new(log_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        Ok(WriteAheadLog {
            writer: BufWriter::new(file),
            log_path: log_path.to_string(),
        })
    }

    pub fn log(&mut self, sql: &str) -> io::Result<()> {
        let command = parse_sql(sql).unwrap();
        let entry = match command {
            SQLCommand::CreateTable { name, columns } => {
                LogEntry::Command(format!("CREATE TABLE {} ({:?})", name, columns))
            }
            SQLCommand::InsertInto { table, values } => {
                LogEntry::Command(format!("INSERT INTO {} VALUES ({:?})", table, values))
            }
            SQLCommand::SelectFrom {
                table,
                page_size,
                page,
                group_by,
                order_by,
            } => LogEntry::SelectFrom {
                table,
                page_size,
                page,
                group_by,
                order_by,
            },
            SQLCommand::DeleteFrom { table, condition } => {
                LogEntry::DeleteFrom { table, condition }
            }
            SQLCommand::Update {
                table,
                assignments,
                condition,
            } => LogEntry::Update {
                table,
                assignments,
                condition,
            },
            SQLCommand::CreateView { name, query, columns } => {
                LogEntry::Command(format!("CREATE VIEW {} AS {} ({})", name, query, columns.join(", ")))
            }
        };

        let entry_str = serde_json::to_string(&entry)?;
        writeln!(self.writer, "{}", entry_str)?;
        self.writer.flush()?;
        Ok(())
    }
}
