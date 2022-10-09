use super::*;
use termbox_sys::*;

impl TermContext {}

pub fn term_init(g: &Galaxy) {
    g.insert_resource(TermContext::new());
}

pub fn term_input(g: &Galaxy) {
    if let Some(_) = g.get_mut_resource::<TermContext>() {
        let mut ev = std::mem::MaybeUninit::uninit();
        let ev = unsafe {
            //  60fps right?
            match tb_peek_event(ev.as_mut_ptr(), 16) {
                0 => return,
                -1 => panic!(),
                _ => ev.assume_init(),
            }
        };
        match ev.etype {
            TB_EVENT_KEY => {
                g.insert_event(TermKeyEvent {
                    key: ev.key,
                    unicode: ev.ch,
                });
            }
            _ => {}
        }
    }
}

//  Render Players and Pipes
pub fn term_render(g: &Galaxy) {
    unsafe { tb_clear() };
    for player in g.query::<&Player>().iter() {
        unsafe {
            tb_change_cell(
                player.0 as i32,
                player.1 as i32,
                '■' as u32,
                TB_DEFAULT,
                TB_DEFAULT,
            )
        }
    }

    for pipe in g.query::<&Pipe>().iter() {
        //  Top
        render_hollow(
            pipe.0 .0 as i32,
            pipe.0 .1 as i32,
            pipe.0 .2 as i32,
            pipe.0 .3 as i32,
        );

        //  Bottom
        render_hollow(
            pipe.1 .0 as i32,
            pipe.1 .1 as i32,
            pipe.1 .2 as i32,
            pipe.1 .3 as i32,
        );
    }

    unsafe { tb_present() };
}

fn render_hollow(x: i32, y: i32, sx: i32, sy: i32) {
    let corners = [(0, 0, '┌'), (sx, 0, '┐'), (0, sy, '└'), (sx, sy, '┘')];

    unsafe {
        //  corners
        for corner in corners.into_iter() {
            tb_change_cell(
                x + corner.0,
                y + corner.1,
                corner.2 as u32,
                TB_DEFAULT,
                TB_DEFAULT,
            );
        }
        //  top horizontal
        for i in 1..sx {
            tb_change_cell(x + i, y, '─' as u32, TB_DEFAULT, TB_DEFAULT);
        }
        //  bottom horizontal
        for i in 1..sx {
            tb_change_cell(x + i, y + sy, '─' as u32, TB_DEFAULT, TB_DEFAULT);
        }
        //  left vertical
        for i in 1..sy {
            tb_change_cell(x, y + i, '│' as u32, TB_DEFAULT, TB_DEFAULT);
        }
        //  right vertical
        for i in 1..sy {
            tb_change_cell(x + sx, y + i, '│' as u32, TB_DEFAULT, TB_DEFAULT);
        }
    }
}
