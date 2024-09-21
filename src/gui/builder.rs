use crate::app_controller::AppController;
use crate::gui_widgets::dialogs::create_table::create_table_dialog;
use crate::gui_widgets::sql::sql_queries;
use crate::gui_widgets::table::list_tables::list_tables;
use crate::gui_widgets::table::table_view::table_view;
use druid::widget::{Button, Flex, Scroll, SizedBox};
use druid::WindowConfig;
use druid::{Widget, WidgetExt};

use crate::FerrousDBState;

pub fn ui_builder() -> impl Widget<FerrousDBState> {
    let create_table_button =
        Button::new("Criar Tabela").on_click(|ctx, data: &mut FerrousDBState, e| {
            // Abrir a janela de diálogo para criar tabela
            let dialog = create_table_dialog();
            let window_config = WindowConfig::default()
                .window_size((400.0, 200.1))
                .transparent(true);
            ctx.new_sub_window(window_config, dialog, data.clone(), e.clone());
        });

    let root = Flex::column()
        .with_child(create_table_button.padding(5.0))
        .with_flex_child(
            SizedBox::new(
                Scroll::new(list_tables())
                    .vertical()
                    .padding(5.0)
                    .expand_height(),
            ),
            2.0,
        )
        .with_flex_child(
            Scroll::new(table_view())
                .vertical()
                .padding(5.0)
                .expand_height(),
            2.0,
        )
        .with_flex_child(
            Scroll::new(Flex::column()).vertical().padding(5.0).expand(),
            1.0,
        )
        .with_child(sql_queries())
        .controller(AppController);

    root
}
