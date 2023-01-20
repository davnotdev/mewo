use super::*;

#[repr(u8)]
#[derive(Default)]
pub enum GpuCompositeLayer {
    #[default]
    Primary = 0,
    Overlay = 10,
    OverlayTop = 20,
    Custom(u8),
    AbsoluteTop = 255,
}

impl GpuContext {
    pub fn compile_sequence(
        &mut self,
        passes: &[GpuPass],
        layer: GpuCompositeLayer,
    ) -> GResult<GpuSequenceId> {
        match self {
            Self::Vulkan(vk) => vk.compile_sequence(passes, layer),
        }
    }
}

pub struct GpuSequenceBuilder {
    passes: Vec<GpuPass>,
}

impl GpuSequenceBuilder {
    pub fn new() -> Self {
        GpuSequenceBuilder { passes: vec![] }
    }

    pub fn pass(&mut self, program: GpuProgramId) -> &mut GpuPass {
        let pass_idx = self.passes.len();
        let new_pass = GpuPass {
            vbos: vec![],
            ibo: None,
            program,
            color_attachments: vec![],
            depth_attachment: None,
            attachment_deps: vec![],

            pass_idx,
        };
        self.passes.push(new_pass);
        self.passes.get_mut(pass_idx).unwrap()
    }

    pub fn get_passes(&self) -> &Vec<GpuPass> {
        &self.passes
    }
}
