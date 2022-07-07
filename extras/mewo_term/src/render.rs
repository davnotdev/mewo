use rust_burrito::*;
use termbox_sys::*;

pub type TermVector2 = (i32, i32);

#[derive(Clone)]
pub enum TermQuadType {
    Dot,
    //  These are Dimensions (x, y)
    Fill(TermVector2),
    Hollow(TermVector2),
}

#[derive(Clone)]
pub struct TermQuad {
    pub bg_color: u16,
    pub fg_color: u16,
    pub qtype: TermQuadType,
    pub position: TermVector2,
}
impl Component for TermQuad {
    fn is_copy() -> bool {
        true
    }
}

impl TermQuad {
    pub fn create(
        position: TermVector2,
        qtype: TermQuadType,
        bg_color: Option<u16>,
        fg_color: Option<u16>,
    ) -> TermQuad {
        TermQuad {
            bg_color: if let Some(c) = bg_color {
                c
            } else {
                TB_DEFAULT
            },
            fg_color: if let Some(c) = fg_color {
                c
            } else {
                TB_DEFAULT
            },
            qtype,
            position,
        }
    }
}

/*
position starts at the top left
  ■    - dot
▐████▌ - fill
▐████▌
┌────┐ - empty
│    │
└────┘
*/

pub fn term_render(_args: SA, wish: Wish<(), &TermQuad, ()>) {
    unsafe { tb_clear() };
    for quad in wish.iter() {
        match quad.qtype {
            TermQuadType::Dot => unsafe {
                tb_change_cell(
                    quad.position.0,
                    quad.position.1,
                    '■' as u32,
                    quad.fg_color,
                    quad.bg_color,
                )
            },
            TermQuadType::Hollow((sx, sy)) => unsafe {
                let sx = sx - 1;
                let sy = sy - 1;
                let corners = [(0, 0, '┌'), (sx, 0, '┐'), (0, sy, '└'), (sx, sy, '┘')];
                //  corners
                for corner in corners.into_iter() {
                    tb_change_cell(
                        quad.position.0 + corner.0,
                        quad.position.1 + corner.1,
                        corner.2 as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  top horizontal
                for i in 1..sx {
                    tb_change_cell(
                        quad.position.0 + i,
                        quad.position.1,
                        '─' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  bottom horizontal
                for i in 1..sx {
                    tb_change_cell(
                        quad.position.0 + i,
                        quad.position.1 + sy,
                        '─' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  left vertical
                for i in 1..sy {
                    tb_change_cell(
                        quad.position.0,
                        quad.position.1 + i,
                        '│' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  right vertical
                for i in 1..sy {
                    tb_change_cell(
                        quad.position.0 + sx,
                        quad.position.1 + i,
                        '│' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
            },
            TermQuadType::Fill((sx, sy)) => unsafe {
                let sx = sx - 1;
                let sy = sy - 1;
                //  left vertical
                for i in 0..=sy {
                    tb_change_cell(
                        quad.position.0,
                        quad.position.1 + i,
                        '▐' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  right vertical
                for i in 0..=sy {
                    tb_change_cell(
                        quad.position.0 + sx,
                        quad.position.1 + i,
                        '▌' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  everything else
                for y in 0..sy + 1 {
                    for x in 1..sx {
                        tb_change_cell(
                            quad.position.0 + x,
                            quad.position.1 + y,
                            '█' as u32,
                            quad.fg_color,
                            quad.bg_color,
                        );
                    }
                }
            },
        }
    }
    unsafe { tb_present() }
}
