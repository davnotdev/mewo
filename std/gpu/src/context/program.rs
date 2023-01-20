use super::*;

// pub enum GpuUniformSet {
//     Fast,
//     MidFast,
//     MidSlow,
//     Slow,
// }

// pub enum GpuUniformType {
//     Buffer,
// }

#[derive(Clone)]
pub enum GpuShaderType {
    Vertex,
    Fragment,
}

pub struct GpuShaderSet<'a>(pub(crate) Vec<(GpuShaderType, &'a [u8])>);

impl<'a> GpuShaderSet<'a> {
    pub fn shaders(shaders: &[(GpuShaderType, &'a [u8])]) -> Self {
        GpuShaderSet(shaders.to_vec())
    }
}

impl GpuContext {
    pub fn new_program(&mut self, shaders: &GpuShaderSet) -> GResult<GpuProgramId> {
        match self {
            GpuContext::Vulkan(vk) => vk.new_program(shaders),
        }
    }
}
