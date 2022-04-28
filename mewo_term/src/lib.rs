// use rustbox::RustBox;
use mewo_ecs::*;

mod input;
mod render;

pub struct TermContext {
    // rb: RustBox,
}
impl Resource for TermContext {}

pub struct TermPlugin;

impl TermPlugin {
    pub fn name() -> &'static str {
        "mewo_tk_term"
    }

    pub fn plugin(pb: &mut PluginBuilder) {
        let mut cmds = pb.commands();
        cmds.modify_resources(|rmgr| {
            rmgr.insert::<TermContext>(TermContext {
   //             rb: RustBox::init(Default::default()).unwrap(),
            })
        });
        pb.dep(mewo_common::EventPlugin::name())
            .dep(mewo_common::TransformPlugin::name());
    }
}
