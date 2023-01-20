use super::error::*;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::vulkan::*;

mod buffer;
mod pass;
mod platform;
mod program;
mod sequence;
mod submit;

macro_rules! def_id_ty {
    ($NAME: ident) => {
        impl $NAME {
            pub fn from_id(id: usize) -> Self {
                Self(id)
            }

            pub fn id(&self) -> usize {
                self.0
            }
        }
    };
}

#[derive(Clone, Copy)]
pub struct GpuVertexBufferId(usize);
#[derive(Clone, Copy)]
pub struct GpuIndexBufferId(usize);
#[derive(Clone, Copy)]
pub struct GpuProgramId(usize);
#[derive(Clone, Copy)]
pub struct GpuSequenceId(usize);

def_id_ty!(GpuVertexBufferId);
def_id_ty!(GpuIndexBufferId);
def_id_ty!(GpuProgramId);
def_id_ty!(GpuSequenceId);

pub enum GpuApi {
    Vulkan,
}

pub enum GpuContext {
    Vulkan(VkContext),
}

pub use buffer::{GpuBufferStorageType, GpuIndexBufferElement, GpuVertexBufferElement};
pub use pass::{
    GpuAttachmentLoadOp, GpuAttachmentReference, GpuAttachmentType, GpuColorAttachment,
    GpuDepthAttachment, GpuPass,
};
pub use program::{GpuShaderSet, GpuShaderType};
pub use sequence::{GpuCompositeLayer, GpuSequenceBuilder};
pub use submit::{
    GpuClearColor, GpuClearDepth, GpuDraw, GpuPassSubmitData, GpuSequenceSubmitData, GpuSubmit,
};
