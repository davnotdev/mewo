use super::*;
use std::collections::HashMap;

pub struct GpuClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub struct GpuClearDepth {
    pub value: f32,
}

pub struct GpuDraw {
    pub first: usize,
    pub count: usize,
}

pub struct GpuPassSubmitData {
    pub(crate) draws: Vec<GpuDraw>,
    pub(crate) draws_indexed: Vec<GpuDraw>,
}

impl GpuPassSubmitData {
    pub fn new() -> Self {
        GpuPassSubmitData {
            draws: vec![],
            draws_indexed: vec![],
        }
    }

    pub fn draw(&mut self, first: usize, count: usize) -> &mut Self {
        self.draws.push(GpuDraw { first, count });
        self
    }

    pub fn draw_indexed(&mut self, first: usize, count: usize) -> &mut Self {
        self.draws_indexed.push(GpuDraw { first, count });
        self
    }
}

pub struct GpuSequenceSubmitData {
    pub(crate) sequence: GpuSequenceId,
    pub(crate) pass_datas: Vec<GpuPassSubmitData>,

    pub(crate) clear_colors: HashMap<GpuAttachmentReference, GpuClearColor>,
    pub(crate) clear_depths: HashMap<GpuAttachmentReference, GpuClearColor>,
}

impl GpuSequenceSubmitData {
    pub fn new(sequence: GpuSequenceId) -> Self {
        GpuSequenceSubmitData {
            sequence,
            pass_datas: vec![],

            clear_colors: HashMap::new(),
            clear_depths: HashMap::new(),
        }
    }

    pub fn pass(&mut self, pass: GpuPassSubmitData) -> &mut Self {
        self.pass_datas.push(pass);
        self
    }

    //  TODO EXT: Validate attachment type.

    pub fn set_attachment_clear_color(
        &mut self,
        attachment_ref: GpuAttachmentReference,
        clear_color: GpuClearColor,
    ) -> &mut Self {
        self.clear_colors.insert(attachment_ref, clear_color);
        self
    }
}

impl GpuContext {
    pub fn submit(&mut self, submit: GpuSubmit) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.submit(submit),
        }
    }
}

#[derive(Default)]
pub struct GpuSubmit<'transfer> {
    pub(crate) should_present: bool,
    pub(crate) sequences: Vec<GpuSequenceSubmitData>,
    pub(crate) vbo_transfers: Vec<(GpuVertexBufferId, &'transfer [GpuVertexBufferElement])>,
    pub(crate) ibo_transfers: Vec<(GpuIndexBufferId, &'transfer [GpuIndexBufferElement])>,
}

impl<'transfer> GpuSubmit<'transfer> {
    pub fn new() -> Self {
        GpuSubmit {
            should_present: false,
            sequences: vec![],
            vbo_transfers: vec![],
            ibo_transfers: vec![],
        }
    }

    pub fn sequence(&mut self, data: GpuSequenceSubmitData) -> &mut Self {
        self.sequences.push(data);
        self
    }

    pub fn present(&mut self) -> &mut Self {
        self.should_present = true;
        self
    }
}
