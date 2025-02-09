use druid::{Widget, WidgetExt, WindowConfig};
use druid::widget::{Button, Flex, Label, Split, CrossAxisAlignment};
use crate::app_controller::AppController;
use crate::gui_widgets::dialogs::create_table::create_table_dialog;
use crate::gui_widgets::sql::sql_queries;
use crate::gui_widgets::table::list_tables::list_tables;
use crate::gui_widgets::table::table_view::table_view;
use crate::theme;
use crate::FerrousDBState;

pub fn ui_builder() -> impl Widget<FerrousDBState> {
    let left_panel = Flex::column()
        .with_child(
            Label::new("FerrousDB")
                .with_text_size(24.0)
                .with_text_color(theme::TEXT_COLOR)
                .padding(theme::PADDING_MEDIUM),
        )
        .with_child(
            Button::new("Nova Tabela")
                .on_click(|ctx, data: &mut FerrousDBState, e| {
                    let dialog = create_table_dialog();
                    let window_config = WindowConfig::default()
                        .window_size((400.0, 200.1))
                        .transparent(true);
                    ctx.new_sub_window(window_config, dialog, data.clone(), e.clone());
                })
                .padding(theme::PADDING_MEDIUM)
                .background(theme::PRIMARY_COLOR)
                .rounded(4.0)
                .fix_width(200.0),
        )
        .with_flex_child(list_tables(), 1.0)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .background(theme::BACKGROUND_COLOR)
        .fix_width(250.0);

    let right_panel = Flex::column()
        .with_flex_child(table_view(), 1.0)
        .with_flex_child(sql_queries(), 1.0)
        .background(theme::BACKGROUND_COLOR);

    Split::columns(left_panel, right_panel)
        .split_point(0.2)
        .bar_size(1.0)
        .solid_bar(true)
        .controller(AppController)
}
