use mewo_common::Event;
use mewo_ecs::*;
use termbox_sys::*;

#[derive(Clone)]
pub struct TermKeyEvent {
    pub unicode: u32,
    pub modifier: u8,
    pub key: u16,
}
impl Component for TermKeyEvent {}
#[derive(Clone)]
pub struct TermResizeEvent {
    pub w: i32,
    pub h: i32,
}
impl Component for TermResizeEvent {}

pub fn term_event(args: &mut SystemArgs, _wish: Wish<(), ()>) {
    let mut ev = std::mem::MaybeUninit::uninit();
    let ev = unsafe {
        //  60fps right?
        let res = tb_peek_event(ev.as_mut_ptr(), 16);
        match res {
            0 => return,
            -1 => std::process::exit(1),
            _ => ev.assume_init(),
        }
    };
    match ev.etype {
        TB_EVENT_KEY => args.cmds.spawn_entity(Some(move |e: &mut EntityWrapper| {
            e.insert_component(Event);
            e.insert_component(TermKeyEvent {
                unicode: ev.ch,
                modifier: ev.emod,
                key: ev.key,
            });
        })),
        TB_EVENT_RESIZE => args.cmds.spawn_entity(Some(move |e: &mut EntityWrapper| {
            e.insert_component(Event);
            e.insert_component(TermResizeEvent { w: ev.w, h: ev.h });
        })),
        _ => {

        }
    }
}
