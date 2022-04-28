/*
use rustbox::Key;
use mewo_common::Event;
use mewo_ecs::*;

#[derive(Clone)]
pub struct TermKeyEvent {
    key: Key,
}
impl Component for TermKeyEvent {}

pub fn term_input(_w: Wish<()>, mut args: SystemArgs) {
    let ctx = args.rmgr.get::<super::TermContext>()
        .unwrap();
    match ctx.rb.poll_event(false).unwrap() {
        rustbox::Event::KeyEvent(key) => {
            args.cmds.spawn_entity()
                .insert_component(Event)
                .insert_component(TermKeyEvent { key });
        },
        _ => {},
    };
}
*/
