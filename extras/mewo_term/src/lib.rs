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
        pb.event::<TermKeyEvent>()
            .comp::<TermQuad>()
            .event::<TermInitEvent>()
            .event::<TermResizeEvent>()
            .resource::<TermContext>()
            .sys(term_event)
            .sys(term_startup)
            .sys(term_render)
    }
}

fn term_startup(mut sb: SystemBus, _: Wish<Startup, (), ()>) {
    sb.resources
        .modify(|mut rcm| rcm.insert(TermContext::create()));
    sb.events.event(TermInitEvent);
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

    pub fn width(&self) -> i32 {
        unsafe { tb_width() }
    }

    pub fn height(&self) -> i32 {
        unsafe { tb_height() }
    }
}

impl Drop for TermContext {
    fn drop(&mut self) {
        unsafe {
            tb_shutdown();
        }
    }
}
