use crate::{core::error_handling::FerrousDBError, core::parser::command::SQLCommand, DataType};
use sqlparser::ast::{Expr, GroupByExpr, Offset, Statement};
use std::collections::HashMap;

use crate::core::table::ColumnSchema;
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
                        let mut page_size = 1000;

                        // Check for LIMIT clause
                        if let Some(limit) = &query.limit {
                            if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = limit {
                                if let Ok(parsed_limit) = n.parse::<usize>() {
                                    page_size = parsed_limit;
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
                                }
                            }
                        }

                        // Parse GROUP BY
                        let group_by = match &select.group_by {
                            GroupByExpr::Expressions(exprs, _modifiers) => {
                                exprs.first().and_then(|expr| {
                                    if let Expr::Identifier(ident) = expr {
                                        Some(ident.value.clone())
                                    } else {
                                        None
                                    }
                                })
                            }
                            _ => None,
                        };

                        // Parse ORDER BY
                        let order_by = query
                            .order_by
                            .as_ref()
                            .and_then(|orders| orders.exprs.first())
                            .and_then(|order| {
                                if let Expr::Identifier(ident) = &order.expr {
                                    Some((ident.value.clone(), !order.asc.unwrap_or(true)))
                                } else {
                                    None
                                }
                            });

                        Ok(SQLCommand::SelectFrom {
                            table: name.to_string(),
                            page_size,
                            page,
                            group_by,
                            order_by,
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
        Statement::Update {
            table,
            assignments,
            selection,
            ..
        } => {
            let table_name = table.to_string();
            let mut update_assignments = HashMap::new();

            for assignment in assignments {
                match &assignment.target {
                    target => {
                        if let Expr::Value(value) = &assignment.value {
                            let column_name = target.to_string();
                            match value {
                                sqlparser::ast::Value::Number(n, _) => {
                                    update_assignments
                                        .insert(column_name, DataType::Integer(n.parse().unwrap()));
                                }
                                sqlparser::ast::Value::SingleQuotedString(s) => {
                                    update_assignments
                                        .insert(column_name, DataType::Text(s.clone()));
                                }
                                _ => {
                                    return Err(FerrousDBError::ParseError(
                                        "Unsupported value type".to_string(),
                                    ))
                                }
                            }
                        }
                    }
                }
            }

            let condition = selection.as_ref().and_then(|expr| {
                if let Expr::BinaryOp { left, op, right } = expr {
                    if let Expr::Identifier(id) = left.as_ref() {
                        if let Expr::Value(value) = right.as_ref() {
                            Some(format!("{}={}", id.value, value))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            Ok(SQLCommand::Update {
                table: table_name,
                assignments: update_assignments,
                condition,
            })
        }
        Statement::Delete(delete) => {
            let table_name = if let Some(using) = &delete.using {
                if let Some(table) = using.first() {
                    table.relation.to_string()
                } else {
                    delete.tables[0].to_string()
                }
            } else {
                delete.tables[0].to_string()
            };

            let condition = delete.selection.as_ref().and_then(|expr| {
                if let Expr::BinaryOp { left, op, right } = expr {
                    if let Expr::Identifier(id) = left.as_ref() {
                        if let Expr::Value(value) = right.as_ref() {
                            Some(format!("{}={}", id.value, value))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

            Ok(SQLCommand::DeleteFrom {
                table: table_name,
                condition,
            })
        }
        Statement::CreateView {
            name,
            columns,
            query,
            ..
        } => {
            let view_name = name.to_string();
            let column_names = columns.iter().map(|c| c.to_string()).collect();
            let view_query = query.to_string();

            Ok(SQLCommand::CreateView {
                name: view_name,
                query: view_query,
                columns: column_names,
            })
        }
        _ => Err(FerrousDBError::ParseError(
            "Unsupported SQL command".to_string(),
        )),
    }
}
