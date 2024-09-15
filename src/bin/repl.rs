use std::io::{self, Write};

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

        match db.execute_sql(input) {
            Ok(result) => {
                println!("{}", result);
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
