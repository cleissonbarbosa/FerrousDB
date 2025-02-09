use druid::{
    widget::{Flex, Label, List, Button},
    Env, Widget, WidgetExt,
};

use crate::{theme, FerrousDBState, SELECT_TABLE};

pub fn list_tables() -> impl Widget<FerrousDBState> {
    // Header for the tables list
    let header = Label::new("Tabelas")
        .with_text_size(theme::TEXT_SIZE_LARGE)
        .with_text_color(theme::TEXT_COLOR)
        .padding(theme::PADDING_MEDIUM);

    // Lista de tabelas com melhor estilo visual
    let table_list = List::new(|| {
        Button::new(|table_name: &String, _e: &Env| table_name.clone())
            .on_click(|ctx, table_name: &mut String, _e| {
                ctx.submit_command(SELECT_TABLE.with(table_name.clone()));
            })
            .background(theme::SURFACE_COLOR)
            .rounded(2.0)
            .padding(theme::PADDING_MEDIUM)
            .expand_width()
    })
    .lens(FerrousDBState::table_names);

    // Container com título e lista
    Flex::column()
        .with_child(header)
        .with_flex_child(
            table_list
                .background(theme::SURFACE_COLOR)
                .rounded(4.0)
                .padding(theme::PADDING_SMALL),
            1.0,
        )
        .background(theme::BACKGROUND_COLOR)
        .padding(theme::PADDING_MEDIUM)
}
