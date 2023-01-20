use super::*;

#[cfg(target_os = "windows")]
fn platform_prefered() -> Vec<GpuApi> {
    vec![GpuApi::Vulkan]
}

#[cfg(target_os = "macos")]
fn platform_prefered() -> Vec<GpuApi> {
    vec![GpuApi::Vulkan]
}

#[cfg(all(not(target_os = "macos"), target_family = "unix"))]
fn platform_prefered() -> Vec<GpuApi> {
    vec![GpuApi::Vulkan]
}

impl GpuContext {
    pub fn new(
        display: &RawDisplayHandle,
        window: &RawWindowHandle,
        w: u32,
        h: u32,
    ) -> Result<Self, Vec<GpuError>>
    where
        Self: Sized,
    {
        let mut fails = vec![];
        for api in platform_prefered() {
            match api {
                GpuApi::Vulkan => match VkContext::new(display, window, w, h) {
                    Ok(context) => return Ok(GpuContext::Vulkan(context)),
                    Err(fail) => fails.push(fail),
                },
            }
        }

        Err(fails)
    }
}
