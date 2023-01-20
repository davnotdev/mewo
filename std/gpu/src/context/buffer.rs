use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GpuBufferStorageType {
    Static,
    Dynamic,
}

pub type GpuVertexBufferElement = f32;
pub type GpuIndexBufferElement = u32;

impl GpuContext {
    pub fn new_vertex_buffer(
        &mut self,
        data: &[GpuVertexBufferElement],
        storage_type: GpuBufferStorageType,
    ) -> GResult<GpuVertexBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_vertex_buffer(data, storage_type),
        }
    }

    pub fn new_index_buffer(
        &mut self,
        data: &[GpuIndexBufferElement],
        storage_type: GpuBufferStorageType,
    ) -> GResult<GpuIndexBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_index_buffer(data, storage_type),
        }
    }
}
