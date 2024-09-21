use std::collections::HashMap;
use std::fmt::format;

use crate::core::error_handling::FerrousDBError;
use crate::core::parser::command::SQLCommand;
use crate::core::table::ColumnSchema;
use crate::{DataType, Row};
use sqlparser::ast::{Expr, Offset, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub fn parse_sql(sql: &str) -> Result<SQLCommand, FerrousDBError> {
    let dialect = GenericDialect {}; // or a more specific dialect if needed
    let ast =
        Parser::parse_sql(&dialect, sql).map_err(|e| FerrousDBError::ParseError(e.to_string()))?;

    if ast.len() != 1 {
        return Err(FerrousDBError::ParseError(
            "Only single SQL statements are supported".to_string(),
        ));
    }

    match &ast[0] {
        Statement::CreateTable(create_table) => {
            let table_name = create_table.name.to_string();
            let column_names: Vec<ColumnSchema> = create_table
                .columns
                .iter()
                .map(|c| ColumnSchema::new(c.name.value.clone(), c.data_type.to_string()))
                .collect();
            Ok(SQLCommand::CreateTable {
                name: table_name,
                columns: column_names,
            })
        }
        Statement::Insert(insert) => {
            // For simplicity, we'll assume a single row of values
            if let Some(query) = &insert.source {
                if let sqlparser::ast::SetExpr::Values(values) = &*query.body {
                    if let Some(row) = values.rows.first() {
                        let mut values: HashMap<String, DataType> =
                            std::collections::HashMap::new();
                        for (col, val) in insert.columns.iter().zip(row.iter()) {
                            values.insert(
                                col.value.clone(),
                                val.to_string().parse::<DataType>().unwrap(),
                            );
                        }
                        Ok(SQLCommand::InsertInto {
                            table: insert.table_name.to_string(),
                            values,
                        })
                    } else {
                        Err(FerrousDBError::ParseError(
                            "No values provided for insert".to_string(),
                        ))
                    }
                } else {
                    Err(FerrousDBError::ParseError(
                        "Unsupported INSERT format".to_string(),
                    ))
                }
            } else {
                Err(FerrousDBError::ParseError(
                    "No values provided for insert".to_string(),
                ))
            }
        }
        Statement::Query(query) => {
            if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
                if let Some(table) = select.from.first() {
                    if let sqlparser::ast::TableFactor::Table { name, .. } = &table.relation {
                        let mut page = 1;
                        let mut page_size = 1000; // default page size

                        // Check for LIMIT clause
                        if let Some(limit) = &query.limit {
                            if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = limit {
                                if let Ok(parsed_limit) = n.parse::<usize>() {
                                    page_size = parsed_limit;
                                } else {
                                    return Err(FerrousDBError::ParseError(
                                        "Invalid number in LIMIT clause".to_string(),
                                    ));
                                }
                            }
                        }

                        // Check for OFFSET clause
                        if let Some(Offset { value, .. }) = &query.offset {
                            if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = value {
                                if let Ok(parsed_offset) = n.parse::<usize>() {
                                    page = (parsed_offset / page_size) + 1;
                                    if parsed_offset % page_size != 0 {
                                        page += 1;
                                    }
                                } else {
                                    return Err(FerrousDBError::ParseError(
                                        "Invalid number in OFFSET clause".to_string(),
                                    ));
                                }
                            }
                        }

                        Ok(SQLCommand::SelectFrom {
                            table: name.to_string(),
                            page_size,
                            page,
                        })
                    } else {
                        Err(FerrousDBError::ParseError(
                            "Unsupported FROM clause".to_string(),
                        ))
                    }
                } else {
                    Err(FerrousDBError::ParseError(
                        "No FROM clause in SELECT statement".to_string(),
                    ))
                }
            } else {
                Err(FerrousDBError::ParseError(
                    "Unsupported query type".to_string(),
                ))
            }
        }
        Statement::Delete(delete) => {
            let table_name = "";

            let condition = match &delete.selection {
                Some(sqlparser::ast::Expr::BinaryOp { left, right, op }) => {
                    let left_column = match *left.clone() {
                        sqlparser::ast::Expr::Identifier(value) => value,
                        _ => {
                            return Err(FerrousDBError::ParseError(
                                "Unsupported condition".to_string(),
                            ))
                        }
                    };

                    let right_value = match *right.clone() {
                        sqlparser::ast::Expr::Value(sqlparser::ast::Value::Number(n, _)) => n,
                        _ => {
                            return Err(FerrousDBError::ParseError(
                                "Unsupported condition".to_string(),
                            ))
                        }
                    };

                    Some(format!("{left_column}{op}{right_value}"))
                }
                _ => Some("".to_string()),
            };

            Ok(SQLCommand::DeleteFrom {
                table: table_name.to_string(),
                condition,
            })
        }

        _ => Err(FerrousDBError::ParseError(
            "Unsupported SQL command".to_string(),
        )),
    }
}
