use std::sync::Arc;

use druid::widget::{Button, Flex, Padding, TextBox};
use druid::{Widget, WidgetExt};
use ferrous_db::ColumnSchema;

use crate::FerrousDBState;

pub fn create_table_dialog() -> impl Widget<FerrousDBState> {
    let table_name = TextBox::new()
        .with_placeholder("Nome da Tabela")
        .lens(FerrousDBState::new_table_name)
        .expand_width();

    let columns = TextBox::new()
        .with_placeholder("Colunas (separadas por vírgula)")
        .lens(FerrousDBState::new_columns)
        .expand_width();

    let create_button = Button::new("Criar").on_click(|ctx, data: &mut FerrousDBState, _| {
        let columns: Vec<ColumnSchema> = data
            .new_columns
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|v| v.parse::<ColumnSchema>().unwrap())
            .collect();
        data.db.create_table(&data.new_table_name, columns).unwrap();

        let mut names = (*data.table_names).clone();
        names.push(data.new_table_name.clone());
        data.table_names = Arc::new(names);

        data.new_table_name.clear();
        data.new_columns.clear();
        ctx.window().close();
    });

    let layout = Flex::column()
        .with_child(table_name.padding(5.0))
        .with_child(columns.padding(5.0))
        .with_child(create_button.padding(5.0));

    Padding::new(10.0, layout)
}
