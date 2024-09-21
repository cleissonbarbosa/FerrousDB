use druid::widget::{Button, Flex};
use druid::{Widget, WidgetExt, WindowConfig};

use crate::FerrousDBState;

use super::dialogs::create_table::create_table_dialog;

pub fn sql_snippets() -> impl Widget<FerrousDBState> {
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
