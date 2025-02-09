use std::sync::Arc;
use druid::widget::{Button, Flex, Padding, TextBox, Label};
use druid::{Widget, WidgetExt};
use crate::{FerrousDBState, theme};

pub fn create_table_dialog() -> impl Widget<FerrousDBState> {
    let table_name = TextBox::new()
        .with_placeholder("Nome da Tabela")
        .lens(FerrousDBState::new_table_name)
        .expand_width()
        .background(theme::SURFACE_COLOR);

    let columns = TextBox::new()
        .with_placeholder("Colunas (separadas por vírgula)")
        .lens(FerrousDBState::new_columns)
        .expand_width()
        .background(theme::SURFACE_COLOR);

    let create_button = Button::new("Criar")
        .on_click(|ctx, data: &mut FerrousDBState, _| {
            let columns_result = data
                .new_columns
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|v| v.parse())
                .collect::<Result<Vec<_>, _>>();

            match columns_result {
                Ok(columns) => {
                    if let Ok(()) = data.db.create_table(&data.new_table_name, columns) {
                        let mut names = (*data.table_names).clone();
                        names.push(data.new_table_name.clone());
                        data.table_names = Arc::new(names);
                        
                        data.new_table_name.clear();
                        data.new_columns.clear();
                        ctx.window().close();
                    }
                }
                Err(_) => {
                    // Em caso de erro, não fazemos nada - poderíamos adicionar um label de erro aqui
                }
            }
        })
        .background(theme::PRIMARY_COLOR);

    let hint = Label::new("Exemplo: nome TEXT, idade INTEGER")
        .with_text_size(theme::TEXT_SIZE_MEDIUM)
        .with_text_color(theme::TEXT_COLOR);

    let layout = Flex::column()
        .with_child(table_name.padding(5.0))
        .with_child(columns.padding(5.0))
        .with_child(hint.padding(5.0))
        .with_child(create_button.padding(5.0))
        .background(theme::BACKGROUND_COLOR);

    Padding::new(10.0, layout)
}
