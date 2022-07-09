use rust_burrito::*;
use termbox_sys::*;

#[derive(Debug, Clone)]
pub struct TermKeyEvent {
    pub unicode: u32,
    pub modifier: u8,
    pub key: u16,
}

impl Event for TermKeyEvent {}

#[derive(Debug, Clone)]
pub struct TermResizeEvent {
    pub w: i32,
    pub h: i32,
}
impl Event for TermResizeEvent {}

pub fn term_event(mut sb: SystemBus, _: Wish<(), (), ()>) {
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
        TB_EVENT_KEY => {
            sb.events.event(TermKeyEvent {
                unicode: ev.ch,
                modifier: ev.emod,
                key: ev.key,
            });
        }
        TB_EVENT_RESIZE => {
            sb.events.event(TermResizeEvent { w: ev.w, h: ev.y });
        }
        _ => {}
    }
}
