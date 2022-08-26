use rust_burrito::*;
use termbox_sys::*;

pub type TermVector2 = (f32, f32);

#[derive(Debug, Clone)]
pub enum TermQuadType {
    Dot,
    //  These are Dimensions (x, y)
    Fill(TermVector2),
    Hollow(TermVector2),
}

impl TermQuadType {
    pub fn width(&self) -> f32 {
        match self {
            Self::Dot => 1.0,
            Self::Fill(v) | Self::Hollow(v) => v.0,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            Self::Dot => 1.0,
            Self::Fill(v) | Self::Hollow(v) => v.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TermQuad {
    pub bg_color: u16,
    pub fg_color: u16,
    pub qtype: TermQuadType,
    pub position: TermVector2,
}
impl Component for TermQuad {}

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

pub fn term_render(sb: SystemBus<&TermQuad, ()>) {
    unsafe { tb_clear() };
    for quad in sb.components.iter() {
        match quad.qtype {
            TermQuadType::Dot => unsafe {
                tb_change_cell(
                    quad.position.0 as i32,
                    quad.position.1 as i32,
                    '■' as u32,
                    quad.fg_color,
                    quad.bg_color,
                )
            },
            TermQuadType::Hollow((sx, sy)) => unsafe {
                let sx = sx as i32 - 1;
                let sy = sy as i32;
                let corners = [(0, 0, '┌'), (sx, 0, '┐'), (0, sy, '└'), (sx, sy, '┘')];
                //  corners
                for corner in corners.into_iter() {
                    tb_change_cell(
                        quad.position.0 as i32 + corner.0,
                        quad.position.1 as i32 + corner.1,
                        corner.2 as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  top horizontal
                for i in 1..sx {
                    tb_change_cell(
                        quad.position.0 as i32 + i,
                        quad.position.1 as i32,
                        '─' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  bottom horizontal
                for i in 1..sx {
                    tb_change_cell(
                        quad.position.0 as i32 + i,
                        quad.position.1 as i32 + sy,
                        '─' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  left vertical
                for i in 1..sy {
                    tb_change_cell(
                        quad.position.0 as i32,
                        quad.position.1 as i32 + i,
                        '│' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  right vertical
                for i in 1..sy {
                    tb_change_cell(
                        quad.position.0 as i32 + sx,
                        quad.position.1 as i32 + i,
                        '│' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
            },
            TermQuadType::Fill((sx, sy)) => unsafe {
                let sx = sx as i32 - 1;
                let sy = sy as i32;
                //  left vertical
                for i in 0..=sy {
                    tb_change_cell(
                        quad.position.0 as i32,
                        quad.position.1 as i32 + i,
                        '▐' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  right vertical
                for i in 0..=sy {
                    tb_change_cell(
                        quad.position.0 as i32 + sx,
                        quad.position.1 as i32 + i,
                        '▌' as u32,
                        quad.fg_color,
                        quad.bg_color,
                    );
                }
                //  everything else
                for y in 0..sy + 1 {
                    for x in 1..sx {
                        tb_change_cell(
                            quad.position.0 as i32 + x,
                            quad.position.1 as i32 + y,
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
