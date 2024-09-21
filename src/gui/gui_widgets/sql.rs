use std::sync::Arc;

use druid::{
    widget::{Button, Flex, Label, TextBox},
    Env, Widget, WidgetExt,
};

use crate::FerrousDBState;

use super::snippets::sql_snippets;

pub fn sql_queries() -> impl Widget<FerrousDBState> {
    let sql_input = TextBox::multiline()
        .with_placeholder("Digite o comando SQL")
        .lens(FerrousDBState::sql_command)
        .fix_height(100.0)
        .expand_width();

    let execute_button =
        Button::new("Executar SQL").on_click(|ctx, data: &mut FerrousDBState, _| {
            match data.db.execute_sql(&data.sql_command) {
                Ok(message) => data.sql_output = message,
                Err(e) => data.sql_output = format!("Erro: {}", e),
            }
            ctx.request_update();
            if data.sql_command.to_uppercase().contains("CREATE TABLE") {
                data.table_names = Arc::new(data.db.tables.clone().into_keys().collect())
            }
        });

    let sql_output =
        Label::new(|data: &FerrousDBState, _env: &Env| data.sql_output.clone()).padding(5.0);

    Flex::column()
        .with_child(sql_snippets().padding(5.0))
        .with_child(sql_input.padding(5.0))
        .with_child(execute_button.padding(5.0))
        .with_child(sql_output.padding(5.0))
}
