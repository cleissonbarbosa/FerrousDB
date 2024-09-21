use std::sync::Arc;

use druid::widget::{Button, Controller, Flex, Label, List, Padding, Scroll, SizedBox, TextBox};
use druid::{
    AppLauncher, Data, Env, Event, EventCtx, Lens, LocalizedString, PlatformError, Widget,
    WidgetExt, WindowDesc,
};
use druid::{Selector, WindowConfig};
use ferrous_db::{ColumnSchema, Row};

#[derive(Clone, Lens, PartialEq)]
struct FerrousDBState {
    /// The number displayed. Generally a valid float.
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

struct AppController;

impl<W: Widget<FerrousDBState>> Controller<FerrousDBState, W> for AppController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FerrousDBState,
        env: &Env,
    ) {
        if let Event::Command(cmd) = event {
            if let Some(table_name) = cmd.get(SELECT_TABLE) {
                println!("selected table: {table_name}");
                data.selected_table = Some(table_name.clone());
                ctx.request_update();
            }
        }
        child.event(ctx, event, data, env);
    }
}

fn ui_builder() -> impl Widget<FerrousDBState> {
    let create_table_button =
        Button::new("Criar Tabela").on_click(|ctx, data: &mut FerrousDBState, e| {
            // Abrir a janela de diálogo para criar tabela
            let dialog = create_table_dialog();
            let window_config = WindowConfig::default()
                .window_size((400.0, 200.1))
                .transparent(true);
            ctx.new_sub_window(window_config, dialog, data.clone(), e.clone());
        });

    let table_list = List::new(|| {
        Label::new(move |table_name: &String, _e: &Env| table_name.clone())
            .on_click(|ctx, table_name: &mut String, _e: &Env| {
                ctx.submit_command(SELECT_TABLE.with(table_name.clone()));
            })
            .padding(5.0)
            .expand_width()
    })
    .lens(FerrousDBState::table_names);

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
            if data.sql_command.to_uppercase().contains("CREATE TABLE") {
                data.table_names = Arc::new(data.db.tables.clone().into_keys().collect())
            }
        });

    let sql_output =
        Label::new(|data: &FerrousDBState, _env: &Env| data.sql_output.clone()).padding(5.0);

    let sql_section = Flex::column()
        .with_child(sql_snippets().padding(5.0))
        .with_child(sql_input.padding(5.0))
        .with_child(execute_button.padding(5.0))
        .with_child(sql_output.padding(5.0));

    let root = Flex::column()
        .with_child(create_table_button.padding(5.0))
        .with_flex_child(
            SizedBox::new(
                Scroll::new(table_list)
                    .vertical()
                    .padding(5.0)
                    .expand_height(),
            ),
            2.0,
        )
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
        .controller(AppController);

    root
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

fn sql_snippets() -> impl Widget<FerrousDBState> {
    let create_button =
        Button::new("CREATE TABLE").on_click(|ctx, data: &mut FerrousDBState, e| {
            let dialog = create_table_dialog();
            let window_config = WindowConfig::default()
                .window_size((400.0, 200.1))
                .transparent(true);
            ctx.new_sub_window(window_config, dialog, data.clone(), e.clone());
        });

    let select_all_button =
        Button::new("SELECT *").on_click(|_ctx, data: &mut FerrousDBState, _env| {
            data.sql_command = "SELECT * FROM ;".to_string();
        });

    let insert_button =
        Button::new("INSERT INTO").on_click(|_ctx, data: &mut FerrousDBState, _env| {
            data.sql_command = "INSERT INTO  (columns) VALUES (values);".to_string();
        });

    let update_button = Button::new("UPDATE").on_click(|_ctx, data: &mut FerrousDBState, _env| {
        data.sql_command = "UPDATE  SET  WHERE ;".to_string();
    });

    let delete_button = Button::new("DELETE").on_click(|_ctx, data: &mut FerrousDBState, _env| {
        data.sql_command = "DELETE FROM  WHERE ;".to_string();
    });

    // Layout para os botões de snippets
    Flex::row()
        .with_child(create_button.padding(5.0))
        .with_child(select_all_button.padding(5.0))
        .with_child(insert_button.padding(5.0))
        .with_child(update_button.padding(5.0))
        .with_child(delete_button.padding(5.0))
}
