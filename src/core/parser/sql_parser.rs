use crate::core::parser::command::SQLCommand;
use sqlparser::ast::{Expr, Offset, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub fn parse_sql(sql: &str) -> Result<SQLCommand, String> {
    let dialect = GenericDialect {}; // or a more specific dialect if needed
    let ast = Parser::parse_sql(&dialect, sql).map_err(|e| e.to_string())?;

    if ast.len() != 1 {
        return Err("Only single SQL statements are supported".to_string());
    }

    match &ast[0] {
        Statement::CreateTable(create_table) => {
            let table_name = create_table.name.to_string();
            let column_names: Vec<String> = create_table
                .columns
                .iter()
                .map(|c| c.name.value.clone())
                .collect();
            let page_size = 10; // default page size
            Ok(SQLCommand::CreateTable {
                name: table_name,
                columns: column_names,
                page_size,
            })
        }
        Statement::Insert(insert) => {
            // For simplicity, we'll assume a single row of values
            if let Some(query) = &insert.source {
                if let sqlparser::ast::SetExpr::Values(values) = &*query.body {
                    if let Some(row) = values.rows.first() {
                        let mut values = std::collections::HashMap::new();
                        for (col, val) in insert.columns.iter().zip(row.iter()) {
                            values.insert(col.value.clone(), val.to_string());
                        }
                        Ok(SQLCommand::InsertInto {
                            table: insert.table_name.to_string(),
                            values,
                        })
                    } else {
                        Err("No values provided for insert".to_string())
                    }
                } else {
                    Err("Unsupported INSERT format".to_string())
                }
            } else {
                Err("No values provided for insert".to_string())
            }
        }
        Statement::Query(query) => {
            if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
                if let Some(table) = select.from.first() {
                    if let sqlparser::ast::TableFactor::Table { name, .. } = &table.relation {
                        let mut page = 1;
                        let mut page_size = 10; // default page size

                        // Check for LIMIT clause
                        if let Some(limit) = &query.limit {
                            if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = limit {
                                page_size = n.parse().unwrap_or(10);
                            }
                        }

                        // Check for OFFSET clause
                        if let Some(Offset { value, .. }) = &query.offset {
                            if let Expr::Value(sqlparser::ast::Value::Number(n, _)) = value {
                                let offset: usize = n.parse().unwrap_or(0);
                                page = (offset / page_size) + 1;
                            }
                        }

                        Ok(SQLCommand::SelectFrom {
                            table: name.to_string(),
                            page,
                        })
                    } else {
                        Err("Unsupported FROM clause".to_string())
                    }
                } else {
                    Err("No FROM clause in SELECT statement".to_string())
                }
            } else {
                Err("Unsupported query type".to_string())
            }
        }
        _ => Err("Unsupported SQL command".to_string()),
    }
}
