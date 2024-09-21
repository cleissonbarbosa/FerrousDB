use druid::{widget::Controller, Env, Event, EventCtx, Widget};

use crate::{FerrousDBState, SELECT_TABLE};

pub struct AppController;

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
