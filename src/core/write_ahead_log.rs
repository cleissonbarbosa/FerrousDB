use std::fs::{File, OpenOptions};
use std::io::BufRead;
use std::io::{BufReader, BufWriter, Write};

use crate::command::SQLCommand;

use super::error_handling::FerrousDBError;

pub struct WriteAheadLog {
    writer: BufWriter<File>,
    log_path: String,
}

impl PartialEq for WriteAheadLog {
    fn eq(&self, other: &Self) -> bool {
        self.log_path == other.log_path
    }
}

impl WriteAheadLog {
    pub fn new(log_path: &str) -> Result<Self, FerrousDBError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        Ok(WriteAheadLog {
            writer: BufWriter::new(file),
            log_path: log_path.to_string(),
        })
    }

    pub fn log(&mut self, entry: &str) -> Result<(), FerrousDBError> {
        let now = chrono::Utc::now();
        writeln!(
            self.writer,
            "[{}] {}",
            now.format("%Y-%m-%d %H:%M:%S"),
            entry
        )?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn recover(&self) -> Result<(), FerrousDBError> {
        // Implement recovery logic
        let file = File::open(self.log_path.clone())?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let entry = line?;
            // Process the log entry
            let command = SQLCommand::from_iter(entry.split_whitespace().map(String::from));
            match command {
                SQLCommand::CreateTable { name, columns } => {
                    // TODO: Implement logic to create table
                }
                SQLCommand::InsertInto { table, values } => {
                    // TODO: Implement logic to insert into table
                }
                SQLCommand::SelectFrom {
                    table,
                    page_size,
                    page,
                } => {
                    // TODO: Implement logic to select from table
                }
                SQLCommand::DeleteFrom { table, condition } => {
                    // TODO: Implement logic to delete from table
                }
                _ => {
                    return Err(FerrousDBError::RecoveryError(
                        "Unsupported SQL command in log entry".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
