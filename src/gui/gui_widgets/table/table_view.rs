use druid::{widget::Label, Env, Widget, WidgetExt};

use crate::FerrousDBState;

pub fn table_view() -> impl Widget<FerrousDBState> {
    // Área para exibir os dados da tabela selecionada
    Label::new(|data: &FerrousDBState, _env: &Env| {
        if let Some(table_name) = &data.selected_table {
            if let Some(table) = data.db.tables.get(table_name) {
                let mut output = String::new();
                for row in &table.rows {
                    output.push_str(&format!("{:?}\n", row.data));
                }
                output
            } else {
                "Tabela não encontrada.".to_string()
            }
        } else {
            "Selecione uma tabela para visualizar os dados.".to_string()
        }
    })
    .padding(5.0)
}
