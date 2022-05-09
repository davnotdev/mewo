use mewo_ecs::*;

pub struct Event;
impl Component for Event {}

fn remove_events(args: &mut SystemArgs, w: Wish<(), With<Event>>) {
    for (e, _event) in w.eiter() {
        args.cmds.remove_entity(e);
    }
}

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn name() -> &'static str {
        "mewo_tk_event"
    }

    fn plugin(a: &mut App) {
        a.component::<Event>().sys(remove_events);
    }
}
