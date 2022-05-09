use mewo_ecs::*;
pub use termbox_sys::*;
use mewo_common::EventPlugin;

mod input;
mod render;

pub use input::{TermKeyEvent, TermResizeEvent};
pub use render::{TermQuad, TermQuadType};

pub struct TermContext;
impl Resource for TermContext {}

impl TermContext {
    fn create() -> TermContext {
        unsafe {
            tb_init();
        };
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
        };
    }
}

pub struct TermPlugin;

impl Plugin for TermPlugin {
    fn name() -> &'static str {
        "mewo_tk_term"
    }

    fn plugin(a: &mut App) {
        let cmds = a.commands();
        cmds.modify_resources(|rmgr| {
            rmgr.insert(TermContext::create());
        });
        a
            .dep(EventPlugin)
            .component::<input::TermKeyEvent>()
            .component::<input::TermResizeEvent>()
            .component::<render::TermQuad>()
            .sys(input::term_event)
            .sys(render::term_render);
    }
}
