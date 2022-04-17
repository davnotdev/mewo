use rustbox::Key;
use mewo_common::Event;
use mewo_ecs::*;

#[derive(Clone)]
pub struct TermKeyEvent {
    key: Key,
}
impl Component for TermKeyEvent {}

pub fn term_input(_w: Wish<()>, args: SystemArgs) {
    let ctx = args.rmgr.get::<super::TermContext>();
    match ctx.rb.poll_event(false).unwrap() {
        rustbox::Event::KeyEvent(key) => {
            args.cmds.spawn_entity(Some(move |mut e: EntityModifier| {
                e.insert_component::<Event>(Event);
                e.insert_component::<TermKeyEvent>(TermKeyEvent { key });
            }));
        },
        _ => {},
    };
}
