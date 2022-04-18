use mewo_ecs::*;

pub struct Event;
impl Component for Event {}

fn remove_events(w: Wish<Read<Event>>, mut args: SystemArgs) {
    for (_event, e) in w.read::<Event>() {
        args.cmds.remove_entity(e);
    }
}

pub struct EventPlugin;

impl EventPlugin {
    pub fn name() -> &'static str {
        "mewo_tk_event"
    }

    pub fn plugin(pb: &mut PluginBuilder) {
        pb
            .component::<Event>()
            .sys(remove_events)
        ;
    }
}

