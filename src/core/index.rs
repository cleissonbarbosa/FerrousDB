use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::DataType;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Index {
    table_name: String,
    column_name: String,
    index_type: IndexType,
    entries: HashMap<DataType, Vec<usize>>,  // Maps values to row indices
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IndexType {
    BTree,
    Hash,
}

impl Index {
    pub fn new(table_name: String, column_name: String, index_type: IndexType) -> Self {
        Index {
            table_name,
            column_name,
            index_type,
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: DataType, row_index: usize) {
        self.entries.entry(value)
            .or_insert_with(Vec::new)
            .push(row_index);
    }

    pub fn find(&self, value: &DataType) -> Option<&Vec<usize>> {
        self.entries.get(value)
    }

    pub fn remove(&mut self, value: &DataType, row_index: usize) {
        if let Some(indices) = self.entries.get_mut(value) {
            indices.retain(|&i| i != row_index);
            if indices.is_empty() {
                self.entries.remove(value);
            }
        }
    }

    pub fn update(&mut self, old_value: &DataType, new_value: DataType, row_index: usize) {
        self.remove(old_value, row_index);
        self.insert(new_value, row_index);
    }
}