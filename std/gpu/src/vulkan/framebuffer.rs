use super::*;

pub struct VkFramebufferExtensions {
    pub color: Option<vk::Format>,
    pub depth: Option<()>,
}

pub struct VkFramebufferColorExtension {
    color_image: VkImage,
    color_image_view: vk::ImageView,

    drop_queue_ref: VkDropQueueRef,
}

impl VkFramebufferColorExtension {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        render_format: vk::Format,
        width: u32,
        height: u32,
    ) -> GResult<Self> {
        let color_image = VkImage::new(
            dev,
            drop_queue_ref,
            alloc,
            render_format,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::Extent3D {
                width,
                height,
                depth: 1,
            },
        )?;
        let color_image_view = new_image_view(
            dev,
            color_image.image,
            render_format,
            vk::ImageAspectFlags::COLOR,
        )?;

        Ok(VkFramebufferColorExtension {
            color_image,
            color_image_view,
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkFramebufferColorExtension {
    fn drop(&mut self) {
        let color_image_view = self.color_image_view;
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_image_view(color_image_view, None)
            }))
    }
}

pub struct VkFramebufferDepthExtension {
    depth_image: VkImage,
    depth_image_view: vk::ImageView,

    drop_queue_ref: VkDropQueueRef,
}

impl VkFramebufferDepthExtension {
    fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        width: u32,
        height: u32,
    ) -> GResult<Self> {
        let depth_image = VkImage::new(
            dev,
            drop_queue_ref,
            alloc,
            vk::Format::D32_SFLOAT,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::Extent3D {
                width,
                height,
                depth: 1,
            },
        )?;
        let depth_image_view = new_image_view(
            dev,
            depth_image.image,
            vk::Format::D32_SFLOAT,
            vk::ImageAspectFlags::DEPTH,
        )?;

        Ok(VkFramebufferDepthExtension {
            depth_image,
            depth_image_view,
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkFramebufferDepthExtension {
    fn drop(&mut self) {
        let depth_image_view = self.depth_image_view;
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_image_view(depth_image_view, None)
            }))
    }
}

pub struct VkLoneFramebuffer {
    pub framebuffer: vk::Framebuffer,

    color: Option<VkFramebufferColorExtension>,
    depth: Option<VkFramebufferDepthExtension>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkLoneFramebuffer {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        render_pass: vk::RenderPass,
        width: usize,
        height: usize,
        exts: VkFramebufferExtensions,
    ) -> GResult<Self> {
        let mut attachments = vec![];

        let color = if let Some(color_format) = exts.color {
            let color = VkFramebufferColorExtension::new(
                dev,
                drop_queue_ref,
                alloc,
                color_format,
                width as u32,
                height as u32,
            )?;
            attachments.push(color.color_image_view);
            Some(color)
        } else {
            None
        };
        let depth = if exts.depth.is_some() {
            let depth = VkFramebufferDepthExtension::new(
                dev,
                drop_queue_ref,
                alloc,
                width as u32,
                height as u32,
            )?;
            attachments.push(depth.depth_image_view);
            Some(depth)
        } else {
            None
        };

        let framebuffer_create = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(width as u32)
            .height(height as u32)
            .layers(1)
            .build();
        let framebuffer = unsafe { dev.create_framebuffer(&framebuffer_create, None) }
            .map_err(|e| gpu_api_err!("vulkan lone framebuffer init {}", e))?;

        Ok(VkLoneFramebuffer {
            framebuffer,
            color,
            depth,
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkLoneFramebuffer {
    fn drop(&mut self) {
        let framebuffer = self.framebuffer;
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_framebuffer(framebuffer, None);
            }))
    }
}

pub struct VkSwapchainFramebufferExtensions {
    depth: Option<VkFramebufferDepthExtension>,
}

pub struct VkSwapchainFramebuffer {
    pub framebuffers: Vec<(vk::Framebuffer, VkSwapchainFramebufferExtensions)>,
    pub has_depth: bool,

    drop_queue_ref: VkDropQueueRef,
}

impl VkSwapchainFramebuffer {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        swapchain: &VkSwapchain,
        render_pass: &vk::RenderPass,
        exts: VkFramebufferExtensions,
    ) -> GResult<VkSwapchainFramebuffer> {
        let framebuffers = swapchain
            .swapchain_image_views
            .iter()
            .map(|&image_view| {
                let mut attachments = vec![image_view];

                let depth = if exts.depth.is_some() {
                    let depth = VkFramebufferDepthExtension::new(
                        dev,
                        drop_queue_ref,
                        alloc,
                        swapchain.extent.width,
                        swapchain.extent.height,
                    )?;
                    attachments.push(depth.depth_image_view);
                    Some(depth)
                } else {
                    None
                };

                let framebuffer_create = vk::FramebufferCreateInfo::builder()
                    .render_pass(*render_pass)
                    .attachments(&attachments)
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(1)
                    .build();
                let framebuffer = unsafe { dev.create_framebuffer(&framebuffer_create, None) }
                    .map_err(|e| gpu_api_err!("vulkan swapchain framebuffer init {}", e))?;
                Ok((framebuffer, VkSwapchainFramebufferExtensions { depth }))
            })
            .collect::<GResult<Vec<_>>>()?;

        Ok(VkSwapchainFramebuffer {
            framebuffers,
            has_depth: exts.depth.is_some(),
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkSwapchainFramebuffer {
    fn drop(&mut self) {
        let framebuffers = self
            .framebuffers
            .iter()
            .map(|(fb, _)| fb)
            .cloned()
            .collect::<Vec<_>>();

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                for framebuffer in framebuffers {
                    dev.destroy_framebuffer(framebuffer, None);
                }
            }))
    }
}
