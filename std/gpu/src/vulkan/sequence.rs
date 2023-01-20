use super::*;

pub enum VkFramebufferVariant {
    Direct(VkSwapchainFramebuffer),
    Composite(VkLoneFramebuffer),
}

impl VkFramebufferVariant {
    pub fn get_current_framebuffer(&self, swapchain_image_idx: u32) -> vk::Framebuffer {
        match self {
            Self::Direct(fb) => fb.framebuffers.get(swapchain_image_idx as usize).unwrap().0,
            Self::Composite(fb) => fb.framebuffer,
        }
    }
}

pub struct VkSequence {
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<vk::Pipeline>,
    pub framebuffer: VkFramebufferVariant,
    layer: GpuCompositeLayer,
    pub passes: Vec<GpuPass>,

    //  Vulkan attachment index => GpuAttachmentReference
    pub attachment_indices: Vec<GpuAttachmentReference>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkContext {
    pub fn compile_sequence(
        &mut self,
        passes: &[GpuPass],
        layer: GpuCompositeLayer,
    ) -> GResult<GpuSequenceId> {
        let swapchain_width = self.swapchain.extent.width;
        let swapchain_height = self.swapchain.extent.height;
        let swapchain_format = self.swapchain.format;

        //  We need to know where each attachment reference maps to in vulkan for clear colors.
        let mut attachment_indices = vec![];

        //  Will we directly render to the swapchain?
        let is_direct = matches!(
            layer,
            GpuCompositeLayer::Primary | GpuCompositeLayer::Custom(0)
        );

        let mut attachments = vec![];
        let mut subpasses = vec![];

        //  Create Subpasses and Write Attachments

        //  Some variables in the below loop don't live long enough.
        let mut each_color_attachment_refs = vec![];
        let mut each_input_attachments = vec![];
        let mut each_depth = vec![];

        for pass in passes.iter() {
            let (mut color_attachment_descs, color_attachment_refs): (Vec<_>, Vec<_>) = pass
                .color_attachments
                .iter()
                .map(|color| {
                    {
                        attachment_indices.push(color.attachment_ref);
                        (
                            vk::AttachmentDescription::builder()
                                .format(swapchain_format)
                                .samples(vk::SampleCountFlags::TYPE_1)
                                .load_op(match color.load {
                                    GpuAttachmentLoadOp::Load => vk::AttachmentLoadOp::LOAD,
                                    GpuAttachmentLoadOp::Clear => vk::AttachmentLoadOp::CLEAR,
                                })
                                .store_op(vk::AttachmentStoreOp::STORE)
                                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                                .initial_layout(vk::ImageLayout::UNDEFINED)
                                //  `is_direct` means that we are rendering to a swapchain.
                                //  `dominant` means that this is the last color attachment we render to before bliting onto the swapchain.
                                .final_layout(if is_direct {
                                    vk::ImageLayout::PRESENT_SRC_KHR
                                } else if color.dominant {
                                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL
                                } else {
                                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                                })
                                .build(),
                            vk::AttachmentReference::builder()
                                .attachment(color.attachment_ref.attachment_idx as u32)
                                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                .build(),
                        )
                    }
                })
                .unzip();
            let depth = pass.depth_attachment.as_ref().map(|depth| {
                let depth_image_format = vk::Format::D32_SFLOAT;

                attachment_indices.push(depth.attachment_ref);

                (
                    vk::AttachmentDescription::builder()
                        .format(depth_image_format)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(match depth.load {
                            GpuAttachmentLoadOp::Load => vk::AttachmentLoadOp::LOAD,
                            GpuAttachmentLoadOp::Clear => vk::AttachmentLoadOp::CLEAR,
                        })
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                        .initial_layout(vk::ImageLayout::UNDEFINED)
                        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .build(),
                    vk::AttachmentReference::builder()
                        .attachment(depth.attachment_ref.attachment_idx as u32)
                        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .build(),
                )
            });

            let input_attachments = pass
                .attachment_deps
                .iter()
                .map(|dep| {
                    vk::AttachmentReference::builder()
                        .attachment(dep.attachment_idx as u32)
                        .layout(match dep.ty {
                            GpuAttachmentType::Color => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            GpuAttachmentType::Depth => {
                                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                            }
                        })
                        .build()
                })
                .collect::<Vec<_>>();

            subpasses.push({
                let partial = vk::SubpassDescription::builder()
                    .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                    .input_attachments(&input_attachments)
                    .color_attachments(&color_attachment_refs);
                let partial = if let Some(depth) = &depth {
                    partial.depth_stencil_attachment(&depth.1)
                } else {
                    partial
                };
                partial.build()
            });

            if let Some(depth) = depth {
                attachments.push(depth.0)
            }

            attachments.append(&mut color_attachment_descs);

            each_input_attachments.push(input_attachments);
            each_color_attachment_refs.push(color_attachment_refs);
            each_depth.push(depth);
        }

        //  Create Subpass Dependencies
        let mut subpass_deps = vec![];
        for (subpass_idx, pass) in passes.iter().enumerate() {
            subpass_deps.append(
                &mut pass
                    .attachment_deps
                    .iter()
                    .filter_map(|dep| {
                        (dep.pass_idx == subpass_idx).then_some({
                            let subpass_dep = vk::SubpassDependency::builder()
                                .src_subpass(dep.pass_idx as u32)
                                .dst_subpass(subpass_idx as u32);
                            let subpass_dep = match dep.ty {
                                GpuAttachmentType::Color => subpass_dep
                                    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                                    .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                                    .src_access_mask(vk::AccessFlags::empty())
                                    .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE),
                                GpuAttachmentType::Depth => subpass_dep
                                    .src_stage_mask(
                                        vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                                            | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                                    )
                                    .dst_stage_mask(
                                        vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                                            | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                                    )
                                    .src_access_mask(vk::AccessFlags::empty())
                                    .dst_access_mask(
                                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                                    ),
                            };
                            subpass_dep.build()
                        })
                    })
                    .collect(),
            );
        }

        //  RenderPass
        let render_pass_create = vk::RenderPassCreateInfo::builder()
            .dependencies(&subpass_deps)
            .attachments(&attachments)
            .subpasses(&subpasses)
            .build();
        let render_pass = unsafe { self.core.dev.create_render_pass(&render_pass_create, None) }
            .map_err(|e| gpu_api_err!("vulkan render pass {}", e))?;

        //  One Framebuffer please!
        let has_color = passes.iter().any(|pass| !pass.color_attachments.is_empty());
        let has_depth = passes.iter().any(|pass| pass.depth_attachment.is_some());

        let framebuffer_ext = VkFramebufferExtensions {
            color: has_color.then_some(swapchain_format),
            depth: has_depth.then_some(()),
        };

        let framebuffer = if is_direct {
            let framebuffer = VkSwapchainFramebuffer::new(
                &self.core.dev,
                &self.drop_queue,
                &mut self.alloc,
                &self.swapchain,
                &render_pass,
                framebuffer_ext,
            )?;
            VkFramebufferVariant::Direct(framebuffer)
        } else {
            let framebuffer = VkLoneFramebuffer::new(
                &self.core.dev,
                &self.drop_queue,
                &mut self.alloc,
                render_pass,
                swapchain_width as usize,
                swapchain_height as usize,
                framebuffer_ext,
            )?;
            VkFramebufferVariant::Composite(framebuffer)
        };

        //  One Pipeline for Each Subpass

        let pipelines = passes
            .iter()
            .enumerate()
            .map(|(subpass_idx, pass)| {
                let program = self.programs.get(pass.program.id()).unwrap();
                program.new_graphics_pipeline(
                    &self.core.dev,
                    render_pass,
                    self.swapchain.extent,
                    subpass_idx,
                )
            })
            .collect::<GResult<Vec<_>>>()?;

        //  Finally Done!
        let sequence = VkSequence {
            render_pass,
            pipelines,
            framebuffer,
            layer,
            passes: passes.to_vec(),
            attachment_indices,

            drop_queue_ref: Arc::clone(&self.drop_queue),
        };
        self.sequences.push(sequence);

        Ok(GpuSequenceId::from_id(self.sequences.len() - 1))
    }
}

impl Drop for VkSequence {
    fn drop(&mut self) {
        let render_pass = self.render_pass;
        let pipelines = self
            .pipelines
            .drain(0..self.pipelines.len())
            .collect::<Vec<_>>();
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_render_pass(render_pass, None);
                pipelines
                    .into_iter()
                    .for_each(|pipeline| dev.destroy_pipeline(pipeline, None))
            }));
    }
}
