use std::sync::Arc;

use builder::ui_builder;
use druid::Selector;
use druid::{AppLauncher, Data, Lens, LocalizedString, PlatformError, WindowDesc};
use ferrous_db::Row;

mod app_controller;
mod builder;
mod gui_widgets;

#[derive(Clone, Lens, PartialEq)]
struct FerrousDBState {
    db: ferrous_db::FerrousDB,
    rows: Vec<Row>,
    new_table_name: String,
    new_columns: String,
    table_names: Arc<Vec<String>>,
    selected_table: Option<String>,
    sql_command: String,
    sql_output: String,
}

impl Data for FerrousDBState {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

const SELECT_TABLE: Selector<String> = Selector::new("app.select-table");

fn main() -> Result<(), PlatformError> {
    let mut data = FerrousDBState {
        db: ferrous_db::FerrousDB::new(),
        rows: Vec::new(),
        new_table_name: "".to_string(),
        new_columns: "".to_string(),
        table_names: Arc::new(Vec::new()),
        selected_table: None,
        sql_command: "".to_string(),
        sql_output: "".to_string(),
    };

    data.table_names = Arc::new(data.db.tables.clone().into_keys().collect());
    let main_window = WindowDesc::new(ui_builder()).title(
        LocalizedString::new("ferrousdb-win-title")
            .with_placeholder("FerrousDB - A Database write in rust"),
    );
    AppLauncher::with_window(main_window)
        .configure_env(|env, _| {
            env.set(
                druid::theme::UI_FONT,
                druid::FontDescriptor::default().with_size(14.0),
            );
        })
        .log_to_console()
        .launch(data)
}
