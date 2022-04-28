use mewo_ecs::*;

pub struct Event;
impl Component for Event {}

fn remove_events(args: &mut SystemArgs, w: Wish<(), With<Event>>) {
    for (e, _event) in w.iter() {
        args.cmds.remove_entity(e);
    }
}

pub struct EventPlugin;

impl EventPlugin {
    pub fn name() -> &'static str {
        "mewo_tk_event"
    }

    pub fn plugin(pb: &mut PluginBuilder) {
        pb.component::<Event>().sys(remove_events);
    }
}
