use std::io::{self, Write};

use ferrous_db::parser_fn::{execute_sql, parse_sql};
use ferrous_db::FerrousDB;

/// Starts a Read-Eval-Print Loop (REPL) for interacting with FerrousDB.
pub fn repl() {
    let mut db = FerrousDB::new();
    loop {
        print!("sql> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        match parse_sql(input) {
            Ok((_, command)) => {
                execute_sql(&mut db, command);
            }
            Err(err) => {
                println!("Error parsing SQL: {:?}", err);
            }
        }
    }
}

fn main() {
    repl();
}
