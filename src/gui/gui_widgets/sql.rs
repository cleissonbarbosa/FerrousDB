use std::sync::Arc;

use druid::{
    widget::{Button, Flex, Label, TextBox},
    Widget, WidgetExt, Env
};

use crate::{theme, FerrousDBState};

use super::snippets::sql_snippets;

pub fn sql_queries() -> impl Widget<FerrousDBState> {
    let header = Label::new("Editor SQL")
        .with_text_size(theme::TEXT_SIZE_LARGE)
        .with_text_color(theme::TEXT_COLOR)
        .padding(theme::PADDING_MEDIUM);

    let sql_input = TextBox::multiline()
        .with_placeholder("Digite seu comando SQL aqui")
        .lens(FerrousDBState::sql_command)
        .fix_height(120.0)
        .expand_width()
        .background(theme::SURFACE_COLOR)
        .border(theme::SECONDARY_COLOR, 1.0)
        .rounded(4.0);

    let execute_button = Button::new("Executar SQL")
        .on_click(|ctx, data: &mut FerrousDBState, _| {
            match data.db.execute_sql(&data.sql_command) {
                Ok(message) => data.sql_output = message,
                Err(e) => data.sql_output = format!("Erro: {}", e),
            }
            ctx.request_update();
            if data.sql_command.to_uppercase().contains("CREATE TABLE") {
                data.table_names = Arc::new(data.db.tables.clone().into_keys().collect())
            }
        })
        .fix_height(theme::BUTTON_HEIGHT)
        .fix_width(theme::BUTTON_WIDTH)
        .padding((theme::PADDING_MEDIUM, theme::PADDING_SMALL))
        .background(theme::PRIMARY_COLOR);

    let output_header = Label::new("Resultado")
        .with_text_size(theme::TEXT_SIZE_MEDIUM)
        .with_text_color(theme::TEXT_COLOR);

    let sql_output = Label::new(|data: &FerrousDBState, _env: &Env| data.sql_output.clone())
        .with_text_size(theme::TEXT_SIZE_MEDIUM)
        .with_text_color(theme::TEXT_COLOR)
        .padding(theme::PADDING_MEDIUM)
        .background(theme::SURFACE_COLOR)
        .rounded(4.0);

    Flex::column()
        .with_child(header)
        .with_child(sql_snippets().padding((0.0, theme::PADDING_MEDIUM)))
        .with_child(sql_input.padding((0.0, theme::PADDING_MEDIUM)))
        .with_child(execute_button)
        .with_child(
            Flex::column()
                .with_child(output_header.padding((0.0, theme::PADDING_MEDIUM)))
                .with_child(sql_output)
                .padding((0.0, theme::PADDING_MEDIUM)),
        )
        .background(theme::BACKGROUND_COLOR)
        .padding(theme::PADDING_LARGE)
}
