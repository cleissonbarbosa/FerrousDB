use druid::{
    widget::{Flex, Label, Scroll},
    Env, Widget, WidgetExt, FontDescriptor, FontFamily,
};

use crate::{theme, FerrousDBState};

pub fn table_view() -> impl Widget<FerrousDBState> {
    // Cabeçalho da tabela
    let header = Flex::row()
        .with_child(
            Label::new("Dados da Tabela")
                .with_text_size(theme::TEXT_SIZE_LARGE)
                .with_text_color(theme::TEXT_COLOR)
                .padding(theme::PADDING_MEDIUM),
        )
        .background(theme::BACKGROUND_COLOR);

    // Área de dados
    let data_view = Label::new(|data: &FerrousDBState, _env: &Env| {
        if let Some(table_name) = &data.selected_table {
            if let Some(table) = data.db.tables.get(table_name) {
                if table.rows.is_empty() {
                    return "Tabela vazia".to_string();
                }

                let mut output = String::new();
                
                // Cabeçalhos das colunas
                if let Some(first_row) = table.rows.first() {
                    for col in first_row.data.keys() {
                        output.push_str(&format!("{:<15} | ", col));
                    }
                    output.push('\n');
                    output.push_str(&"-".repeat(output.len()));
                    output.push('\n');
                }

                // Dados
                for row in &table.rows {
                    for value in row.data.values() {
                        output.push_str(&format!("{:<15} | ", value.get_value()));
                    }
                    output.push('\n');
                }
                output
            } else {
                "Tabela não encontrada".to_string()
            }
        } else {
            "Selecione uma tabela para visualizar os dados".to_string()
        }
    })
    .with_text_size(theme::TEXT_SIZE_MEDIUM)
    .with_font(FontDescriptor::new(FontFamily::MONOSPACE))
    .with_text_color(theme::TEXT_COLOR)
    .padding(theme::PADDING_MEDIUM);

    Flex::column()
        .with_child(header)
        .with_flex_child(
            Scroll::new(data_view)
                .vertical()
                .background(theme::SURFACE_COLOR)
                .rounded(4.0)
                .padding(theme::PADDING_SMALL),
            1.0,
        )
        .background(theme::BACKGROUND_COLOR)
        .padding(theme::PADDING_MEDIUM)
}
