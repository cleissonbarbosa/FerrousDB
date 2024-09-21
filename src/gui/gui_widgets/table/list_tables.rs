use druid::{
    widget::{Label, List},
    Env, Widget, WidgetExt,
};

use crate::{FerrousDBState, SELECT_TABLE};

pub fn list_tables() -> impl Widget<FerrousDBState> {
    List::new(|| {
        Label::new(move |table_name: &String, _e: &Env| table_name.clone())
            .on_click(|ctx, table_name: &mut String, _e: &Env| {
                ctx.submit_command(SELECT_TABLE.with(table_name.clone()));
            })
            .padding(5.0)
            .expand_width()
    })
    .lens(FerrousDBState::table_names)
}
