use serde::{Deserialize, Serialize};

use super::{error_handling::FerrousDBError, row::Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub name: String,
    pub query: String,
    pub columns: Vec<String>,
}

impl View {
    pub fn new(name: String, query: String, columns: Vec<String>) -> Self {
        View {
            name,
            query,
            columns,
        }
    }
}

#[derive(Debug)]
pub enum ViewResult<'a> {
    Success(Vec<&'a Row>),
    Error(FerrousDBError),
}