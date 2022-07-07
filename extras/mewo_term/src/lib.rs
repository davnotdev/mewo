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
            .event::<TermResizeEvent>()
            .resource::<TermContext>()
            .sys(term_event)
            .sys(term_startup)
            .sys(term_render)
    }
}

fn term_startup(mut args: SA, _wish: Wish<Startup, (), ()>) {
    args.resources
        .modify(|mut rcm| rcm.insert(TermContext::create()))
}

#[derive(Clone)]
pub struct TermContext;

impl Resource for TermContext {}

impl TermContext {
    pub fn create() -> TermContext {
        unsafe { tb_init() };
        TermContext
    }
}

impl Drop for TermContext {
    fn drop(&mut self) {
        unsafe {
            tb_shutdown();
        }
    }
}
