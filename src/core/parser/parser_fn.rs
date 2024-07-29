use std::collections::HashMap;

use crate::FerrousDB;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::command::SQLCommand;

pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

pub fn parse_create_table(input: &str) -> IResult<&str, SQLCommand> {
    let (input, _) = tag("CREATE TABLE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, columns) = delimited(
        char('('),
        separated_list0(
            preceded(multispace0, char(',')),
            preceded(multispace0, parse_identifier),
        ),
        char(')'),
    )(input)?;
    Ok((
        input,
        SQLCommand::CreateTable {
            name: name.to_string(),
            columns: columns.into_iter().map(|s| s.to_string()).collect(),
        },
    ))
}

pub fn parse_insert_into(input: &str) -> IResult<&str, SQLCommand> {
    let (input, _) = tag("INSERT INTO")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, table) = parse_identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, columns) = delimited(
        char('('),
        separated_list0(
            preceded(multispace0, char(',')),
            preceded(multispace0, parse_identifier)
        ),
        char(')'),
    )(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("VALUES")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, values) = delimited(
        char('('),
        separated_list0(
            preceded(multispace0, char(',')),
            preceded(multispace0, parse_identifier)
        ),
        char(')'),
    )(input)?;
    if columns.len() != values.len() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    let values: HashMap<_, _> = columns.into_iter().zip(values.into_iter()).collect();
    Ok((
        input,
        SQLCommand::InsertInto {
            table: table.to_string(),
            values: values.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        },
    ))
}

pub fn parse_select_from(input: &str) -> IResult<&str, SQLCommand> {
    let (input, _) = tag("SELECT * FROM")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, table) = parse_identifier(input)?;
    Ok((
        input,
        SQLCommand::SelectFrom {
            table: table.to_string(),
        },
    ))
}

pub fn parse_sql(input: &str) -> IResult<&str, SQLCommand> {
    alt((parse_create_table, parse_insert_into, parse_select_from))(input)
}

pub fn execute_sql(db: &mut FerrousDB, command: SQLCommand) {
    match command {
        SQLCommand::CreateTable { name, columns } => {
            db.create_table(&name, columns.iter().map(|s| s.as_str()).collect());
        }
        SQLCommand::InsertInto { table, values } => {
            db.insert_into(&table, values);
        }
        SQLCommand::SelectFrom { table } => {
            if let Some(table) = db.select_from(&table) {
                println!("Table: {:?}", table);
            } else {
                println!("Table not found");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse_identifier("abc"), Ok(("", "abc")));
        assert_eq!(parse_identifier("abc123"), Ok(("", "abc123")));
        assert_eq!(parse_identifier("abc_123"), Ok(("", "abc_123")));
        assert_eq!(
            parse_identifier("123abc"),
            Err(nom::Err::Error(nom::error::Error::new(
                "123abc",
                nom::error::ErrorKind::TakeWhile1
            )))
        );
        assert_eq!(
            parse_identifier("123"),
            Err(nom::Err::Error(nom::error::Error::new(
                "123",
                nom::error::ErrorKind::TakeWhile1
            )))
        );
    }

    #[test]
    fn test_parse_create_table() {
        assert_eq!(
            parse_create_table("CREATE TABLE users (name, age)"),
            Ok((
                "",
                SQLCommand::CreateTable {
                    name: "users".to_string(),
                    columns: vec!["name".to_string(), "age".to_string()]
                }
            ))
        );
    }

    #[test]
    fn test_parse_insert_into() {
        assert_eq!(
            parse_insert_into("INSERT INTO users (name, age) VALUES (Alice, 30)"),
            Ok((
                "",
                SQLCommand::InsertInto {
                    table: "users".to_string(),
                    values: {
                        let mut map = std::collections::HashMap::new();
                        map.insert("name".to_string(), "Alice".to_string());
                        map.insert("age".to_string(), "30".to_string());
                        map
                    }
                }
            ))
        );
    }

    #[test]
    fn test_parse_select_from() {
        assert_eq!(
            parse_select_from("SELECT * FROM users"),
            Ok((
                "",
                SQLCommand::SelectFrom {
                    table: "users".to_string()
                }
            ))
        );
    }
}
