use super::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum GpuAttachmentType {
    Color,
    Depth,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct GpuAttachmentReference {
    pub(crate) attachment_idx: usize,
    pub(crate) pass_idx: usize,
    pub(crate) ty: GpuAttachmentType,
}

#[derive(Clone)]
pub enum GpuAttachmentLoadOp {
    Load,
    Clear,
}

#[derive(Clone)]
pub struct GpuColorAttachment {
    pub(crate) dominant: bool,
    pub(crate) attachment_ref: GpuAttachmentReference,
    pub(crate) load: GpuAttachmentLoadOp,
}

#[derive(Clone)]
pub struct GpuDepthAttachment {
    pub(crate) attachment_ref: GpuAttachmentReference,
    pub(crate) load: GpuAttachmentLoadOp,
}

#[derive(Clone)]
pub struct GpuPass {
    pub(crate) vbos: Vec<GpuVertexBufferId>,
    pub(crate) ibo: Option<GpuIndexBufferId>,
    pub(crate) program: GpuProgramId,

    pub(crate) color_attachments: Vec<GpuColorAttachment>,
    pub(crate) depth_attachment: Option<GpuDepthAttachment>,
    pub(crate) attachment_deps: Vec<GpuAttachmentReference>,

    pub(crate) pass_idx: usize,
}

impl GpuPass {
    pub fn add_vertex_buffer(&mut self, vbo: GpuVertexBufferId) {
        self.vbos.push(vbo);
    }

    pub fn set_index_buffer(&mut self, ibo: GpuIndexBufferId) {
        self.ibo = Some(ibo);
    }

    pub fn read(&mut self, dep: GpuAttachmentReference) {
        self.attachment_deps.push(dep);
    }

    pub fn write_color(
        &mut self,
        attachment_idx: usize,
        dominant: bool,
        load: GpuAttachmentLoadOp,
    ) -> GpuAttachmentReference {
        let self_attachment_ref = GpuAttachmentReference {
            attachment_idx,
            pass_idx: self.pass_idx,
            ty: GpuAttachmentType::Color,
        };
        self.color_attachments.push(GpuColorAttachment {
            dominant,
            attachment_ref: self_attachment_ref,
            load,
        });
        self_attachment_ref
    }
}
