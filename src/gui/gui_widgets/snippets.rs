use druid::widget::{Button, Flex};
use druid::{Widget, WidgetExt, WindowConfig};

use crate::{FerrousDBState, theme};

use super::dialogs::create_table::create_table_dialog;

pub fn sql_snippets() -> impl Widget<FerrousDBState> {
    fn button_style<W: Widget<FerrousDBState> + 'static>(button: W) -> impl Widget<FerrousDBState> {
        button
            .fix_height(theme::BUTTON_HEIGHT)
            .background(theme::SURFACE_COLOR)
            .rounded(4.0)
            .border(theme::PRIMARY_COLOR, 1.0)
            .padding((theme::PADDING_SMALL, 0.0))
    }

    let create_button = button_style(
        Button::new("CREATE TABLE")
            .on_click(|ctx, data: &mut FerrousDBState, e| {
                let dialog = create_table_dialog();
                let window_config = WindowConfig::default()
                    .window_size((400.0, 200.1))
                    .transparent(true);
                ctx.new_sub_window(window_config, dialog, data.clone(), e.clone());
            })
    );

    let select_all_button = button_style(
        Button::new("SELECT *")
            .on_click(|_ctx, data: &mut FerrousDBState, _env| {
                data.sql_command = "SELECT * FROM ;".to_string();
            })
    );

    let insert_button = button_style(
        Button::new("INSERT INTO")
            .on_click(|_ctx, data: &mut FerrousDBState, _env| {
                data.sql_command = "INSERT INTO  (columns) VALUES (values);".to_string();
            })
    );

    let update_button = button_style(
        Button::new("UPDATE")
            .on_click(|_ctx, data: &mut FerrousDBState, _env| {
                data.sql_command = "UPDATE  SET  WHERE ;".to_string();
            })
    );

    let delete_button = button_style(
        Button::new("DELETE")
            .on_click(|_ctx, data: &mut FerrousDBState, _env| {
                data.sql_command = "DELETE FROM  WHERE ;".to_string();
            })
    );

    Flex::row()
        .with_child(create_button)
        .with_spacer(theme::PADDING_SMALL)
        .with_child(select_all_button)
        .with_spacer(theme::PADDING_SMALL)
        .with_child(insert_button)
        .with_spacer(theme::PADDING_SMALL)
        .with_child(update_button)
        .with_spacer(theme::PADDING_SMALL)
        .with_child(delete_button)
        .must_fill_main_axis(false)
}
