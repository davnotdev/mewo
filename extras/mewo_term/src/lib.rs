use rust_burrito::*;
use termbox_sys::*;

mod input;
mod render;

pub use input::*;
pub use render::*;

pub struct TermPlugin;
impl Plugin for TermPlugin {
    fn name() -> &'static str {
        "mewo_extras_term"
    }

    fn plugin(pb: PluginBuilder) -> PluginBuilder {
        pb.bootstrap(term_startup)
            .update(term_event)
            .update(term_render)
    }
}

fn term_startup(mut sb: SystemBus<(), ()>) -> Option<()> {
    sb.resources.insert(TermContext::create());
    sb.events.event(TermInitEvent);
    Some(())
}

#[derive(Clone)]
pub struct TermInitEvent;
impl Event for TermInitEvent {}

#[derive(Clone)]
pub struct TermContext;

impl Resource for TermContext {}

impl TermContext {
    pub fn create() -> TermContext {
        unsafe { tb_init() };
        TermContext
    }

    pub fn width(&self) -> f32 {
        unsafe { tb_width() as f32 }
    }

    pub fn height(&self) -> f32 {
        unsafe { tb_height() as f32 }
    }
}

impl Drop for TermContext {
    fn drop(&mut self) {
        unsafe {
            tb_shutdown();
        }
    }
}
