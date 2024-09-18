use druid::widget::{Button, Flex, Label, List, Padding, Scroll, TextBox};
use druid::{
    AppLauncher, Data, Env, EventCtx, Key, Lens, LocalizedString, PlatformError, Widget, WidgetExt,
    WindowDesc,
};
use druid::{LensExt, WindowConfig};
use ferrous_db::{FerrousDB, Row};

#[derive(Clone, Lens, PartialEq)]
struct FerrousDBState {
    /// The number displayed. Generally a valid float.
    db: ferrous_db::FerrousDB,
    rows: Vec<Row>,
    new_table_name: String,
    new_columns: String,
    table_names: im::Vector<String>,
    selected_table: Option<String>,
    sql_command: String,
    sql_output: String,
}

impl Data for FerrousDBState {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder()).title(
        LocalizedString::new("ferrousdb-win-title")
            .with_placeholder("FerrousDB - A Database write in rust"),
    );
    let data = FerrousDBState {
        db: ferrous_db::FerrousDB::new(),
        rows: Vec::new(),
        new_table_name: "".to_string(),
        new_columns: "".to_string(),
        table_names: im::Vector::new(),
        selected_table: None,
        sql_command: "".to_string(),
        sql_output: "".to_string(),
    };
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

fn ui_builder() -> impl Widget<FerrousDBState> {
    let create_table_button =
        Button::new("Criar Tabela").on_click(|ctx, data: &mut FerrousDBState, e| {
            // Abrir a janela de diálogo para criar tabela
            let dialog = create_table_dialog();
            let window_config = WindowConfig::default().window_size((400.0, 200.1)).transparent(true);
            ctx.new_sub_window(window_config, dialog, data.clone(), e.clone());
        });
    //let insert_button =
     //   Button::new("Inserir Tabela").on_click(|ctx, data: &mut FerrousDBState, _| {
            // Lógica para inserir uma nova tabela
    //        let new_table_name = format!("Tabela_{}", data.db.tables.len() + 1);
     //       data.db.create_table(&new_table_name, vec![]);
    //        ctx.request_update(); // Atualiza a interface
    //    });

    //let table_list = List::new(|| {
    //   Label::new(|table_name: &String, _env: &Env| table_name.clone())
    //        .on_click(|ctx, data: &mut FerrousDBState, e: &Env| {
    //            data.selected_table = e.get("key");
    // Atualize a visualização dos dados da tabela selecionada
    //             ctx.request_update();
    //        })
    //          .padding(5.0)
    //        .expand_width()
    //})
    //.lens(FerrousDBState::table_names);

    // Área para exibir os dados da tabela selecionada
    let table_data = Label::new(|data: &FerrousDBState, _env: &Env| {
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
    .padding(5.0);

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
        });

    let sql_output =
        Label::new(|data: &FerrousDBState, _env: &Env| data.sql_output.clone()).padding(5.0);

    let sql_section = Flex::column()
        .with_child(sql_input.padding(5.0))
        .with_child(execute_button.padding(5.0))
        .with_child(sql_output.padding(5.0));

    Flex::column()
        .with_child(create_table_button.padding(5.0))
        //.with_child(
        //    Scroll::new(table_list)
        //        .vertical()
        //         .padding(5.0)
        //       .expand_height(),
        // )
        .with_flex_child(
            Scroll::new(table_data)
                .vertical()
                .padding(5.0)
                .expand_height(),
            2.0,
        )
        .with_flex_child(
            Scroll::new(Flex::column()).vertical().padding(5.0).expand(),
            1.0,
        )
        .with_child(sql_section)
}

fn create_table_dialog() -> impl Widget<FerrousDBState> {
    let table_name = TextBox::new()
        .with_placeholder("Nome da Tabela")
        .lens(FerrousDBState::new_table_name)
        .expand_width();

    let columns = TextBox::new()
        .with_placeholder("Colunas (separadas por vírgula)")
        .lens(FerrousDBState::new_columns)
        .expand_width();

    let create_button = Button::new("Criar").on_click(|ctx, data: &mut FerrousDBState, _| {
        let columns: Vec<&str> = data
            .new_columns
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        data.db.create_table(&data.new_table_name, columns);
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
