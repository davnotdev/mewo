use super::*;
use std::mem::ManuallyDrop;

//  How to synchronize.
//
//  |wait> frame_fence |reset>
//
//  aquire_image()
//      |signal> image_aquire_semaphore
//
//  image_aquire_semaphore
//      |wait> transfer()
//      |barrier> render()
//      |barrier> composite()
//      |signal> [render_semaphore, frame_fence]
//
//  render_semaphore
//      |wait> present()
//

pub struct VkSubmitData {
    frame_fence: ManuallyDrop<VkFrameDependent<vk::Fence>>,
    render_semaphore: ManuallyDrop<VkFrameDependent<vk::Semaphore>>,
    image_aquire_semaphore: ManuallyDrop<VkFrameDependent<vk::Semaphore>>,

    graphics_command_buffer: VkFrameDependent<vk::CommandBuffer>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkSubmitData {
    pub fn new(
        dev: &Device,
        frame: &VkFrame,
        graphics_command_pool: vk::CommandPool,
        drop_queue_ref: &VkDropQueueRef,
    ) -> GResult<Self> {
        let frame_fence = ManuallyDrop::new(VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| new_fence(dev, true))
                .collect::<GResult<Vec<_>>>()?,
        ));
        let render_semaphore = ManuallyDrop::new(VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| new_semaphore(dev))
                .collect::<GResult<Vec<_>>>()?,
        ));
        let image_aquire_semaphore = ManuallyDrop::new(VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| new_semaphore(dev))
                .collect::<GResult<Vec<_>>>()?,
        ));
        let graphics_command_buffer = VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| {
                    let command_buffer_alloc = vk::CommandBufferAllocateInfo::builder()
                        .command_pool(graphics_command_pool)
                        .command_buffer_count(1)
                        .build();
                    Ok(unsafe {
                        dev.allocate_command_buffers(&command_buffer_alloc)
                            .map_err(|e| gpu_api_err!("vulkan submit new command buffer {}", e))?[0]
                    })
                })
                .collect::<GResult<Vec<_>>>()?,
        );
        Ok(VkSubmitData {
            frame_fence,
            render_semaphore,
            image_aquire_semaphore,
            graphics_command_buffer,
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkSubmitData {
    fn drop(&mut self) {
        let frame_fence = unsafe { ManuallyDrop::take(&mut self.frame_fence).take_all() };
        let render_semaphore = unsafe { ManuallyDrop::take(&mut self.render_semaphore).take_all() };
        let image_aquire_semaphore =
            unsafe { ManuallyDrop::take(&mut self.image_aquire_semaphore).take_all() };

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                frame_fence
                    .into_iter()
                    .for_each(|fence| dev.destroy_fence(fence, None));
                render_semaphore
                    .into_iter()
                    .for_each(|semaphore| dev.destroy_semaphore(semaphore, None));
                image_aquire_semaphore
                    .into_iter()
                    .for_each(|semaphore| dev.destroy_semaphore(semaphore, None));
            }));
    }
}

impl VkContext {
    pub fn submit(&mut self, submit: GpuSubmit) -> GResult<()> {
        let frame_fence = *self.submit.frame_fence.get(&self.frame);
        let render_semaphore = *self.submit.render_semaphore.get(&self.frame);
        let image_aquire_semaphore = *self.submit.image_aquire_semaphore.get(&self.frame);
        let graphics_command_buffer = *self.submit.graphics_command_buffer.get(&self.frame);

        //  TODO FIX: Replace unwraps.
        unsafe {
            let (swapchain_image_index, _suboptimal) =
                match self.swapchain.swapchain_ext.acquire_next_image(
                    self.swapchain.swapchain,
                    std::u64::MAX,
                    image_aquire_semaphore,
                    vk::Fence::null(),
                ) {
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        todo!("resize");
                        return Ok(());
                    }
                    Err(e) => Err(gpu_api_err!("vulkan aquire image {}", e))?,
                    Ok(ret) => ret,
                };

            self.core
                .dev
                .wait_for_fences(&[frame_fence], true, std::u64::MAX)
                .unwrap();
            self.core.dev.reset_fences(&[frame_fence]).unwrap();

            self.core
                .dev
                .reset_command_buffer(
                    graphics_command_buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap();

            let command_create = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            self.core
                .dev
                .begin_command_buffer(graphics_command_buffer, &command_create)
                .unwrap();

            //  Graphics Related Transfers
            submit.vbo_transfers.iter().for_each(|(vbo, data)| {
                let vbo = self.vbos.get(vbo.id()).unwrap();
                vbo.buffer
                    .map_copy_data(
                        data.as_ptr() as *const u8,
                        data.len() * std::mem::size_of::<GpuVertexBufferElement>(),
                    )
                    .unwrap();
            });

            submit.ibo_transfers.iter().for_each(|(ibo, data)| {
                let ibo = self.ibos.get(ibo.id()).unwrap();
                ibo.buffer
                    .map_copy_data(
                        data.as_ptr() as *const u8,
                        data.len() * std::mem::size_of::<GpuIndexBufferElement>(),
                    )
                    .unwrap();
            });

            //  Read somewhere that this is actually unneccessary.
            let graphics_memory_barrier = vk::MemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::HOST_WRITE)
                .dst_access_mask(
                    vk::AccessFlags::INDEX_READ
                        | vk::AccessFlags::VERTEX_ATTRIBUTE_READ
                        | vk::AccessFlags::UNIFORM_READ
                        | vk::AccessFlags::SHADER_READ
                        | vk::AccessFlags::SHADER_WRITE
                        | vk::AccessFlags::TRANSFER_READ
                        | vk::AccessFlags::TRANSFER_WRITE,
                )
                .build();
            self.core.dev.cmd_pipeline_barrier(
                graphics_command_buffer,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[graphics_memory_barrier],
                &[],
                &[],
            );

            //  TODO: Render
            for (seq_idx, seq_data) in submit.sequences.iter().enumerate() {
                let sequence = self.sequences.get(seq_data.sequence.id()).unwrap();

                //  Clear Values
                let mut clear_values =
                    vec![vk::ClearValue::default(); sequence.attachment_indices.len()];

                fn attachment_ref_to_index(
                    sequence: &VkSequence,
                    attachment_ref: GpuAttachmentReference,
                ) -> usize {
                    sequence
                        .attachment_indices
                        .iter()
                        .position(|&f| f == attachment_ref)
                        .unwrap()
                }

                for (&attachment_ref, clear) in seq_data.clear_colors.iter() {
                    let idx = attachment_ref_to_index(sequence, attachment_ref);
                    clear_values[idx] = vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [clear.r, clear.g, clear.b, clear.a],
                        },
                    };
                }

                //  Rendering Time!

                let render_pass_begin = vk::RenderPassBeginInfo::builder()
                    .render_pass(sequence.render_pass)
                    .clear_values(&clear_values)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swapchain.extent,
                    })
                    .framebuffer(
                        sequence
                            .framebuffer
                            .get_current_framebuffer(swapchain_image_index),
                    )
                    .build();
                self.core.dev.cmd_begin_render_pass(
                    graphics_command_buffer,
                    &render_pass_begin,
                    vk::SubpassContents::INLINE,
                );
                for (pass_idx, pass) in sequence.passes.iter().enumerate() {
                    //  Index Buffer
                    if let Some(ibo) = pass.ibo {
                        let ibo = self.ibos.get(ibo.id()).unwrap();
                        self.core.dev.cmd_bind_index_buffer(
                            graphics_command_buffer,
                            ibo.buffer.buffer,
                            0,
                            match std::mem::size_of::<GpuIndexBufferElement>() {
                                4 => vk::IndexType::UINT32,
                                2 => vk::IndexType::UINT16,
                                _ => unimplemented!("vulkan bad GpuIndexBufferElement type"),
                            },
                        )
                    }

                    //  Vertex Buffers
                    let vbo_buffers = pass
                        .vbos
                        .iter()
                        .map(|vbo| {
                            Ok(self
                                .vbos
                                .get(vbo.id())
                                .ok_or(gpu_api_err!("vulkan bad vbo ({})", vbo.id()))?
                                .buffer
                                .buffer)
                        })
                        .collect::<GResult<Vec<_>>>()?;
                    let vbo_offsets = (0..pass.vbos.len()).map(|_| 0).collect::<Vec<_>>();
                    self.core.dev.cmd_bind_vertex_buffers(
                        graphics_command_buffer,
                        0,
                        &vbo_buffers,
                        &vbo_offsets,
                    );

                    //  Program
                    self.core.dev.cmd_bind_pipeline(
                        graphics_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        sequence.pipelines[pass_idx],
                    );

                    //  Draw
                    for pass in seq_data.pass_datas.iter() {
                        for draw in pass.draws.iter() {
                            self.core.dev.cmd_draw(
                                graphics_command_buffer,
                                draw.count as u32,
                                1,
                                draw.first as u32,
                                0,
                            );
                        }
                        for draw_indexed in pass.draws_indexed.iter() {
                            self.core.dev.cmd_draw_indexed(
                                graphics_command_buffer,
                                draw_indexed.count as u32,
                                1,
                                draw_indexed.first as u32,
                                0,
                                0,
                            );
                        }
                    }

                    //  Progress
                    if seq_idx != submit.sequences.len() - 1 {
                        self.core
                            .dev
                            .cmd_next_subpass(graphics_command_buffer, vk::SubpassContents::INLINE);
                    }
                }
                self.core.dev.cmd_end_render_pass(graphics_command_buffer);
            }

            //  TODO Composite

            self.core
                .dev
                .end_command_buffer(graphics_command_buffer)
                .unwrap();

            let mut submit_signal_semaphores = vec![];
            if submit.should_present {
                submit_signal_semaphores.push(render_semaphore);
            }
            let submit_create = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .wait_semaphores(&[image_aquire_semaphore])
                .signal_semaphores(&submit_signal_semaphores)
                .command_buffers(&[graphics_command_buffer])
                .build();

            self.core
                .dev
                .queue_submit(self.core.graphics_queue, &[submit_create], frame_fence)
                .unwrap();

            self.frame.advance_frame();

            if submit.should_present {
                let present_create = vk::PresentInfoKHR::builder()
                    .wait_semaphores(&[render_semaphore])
                    .swapchains(&[self.swapchain.swapchain])
                    .image_indices(&[swapchain_image_index])
                    .build();

                match self
                    .swapchain
                    .swapchain_ext
                    .queue_present(self.core.graphics_queue, &present_create)
                {
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        todo!("resize")
                    }
                    Err(e) => Err(gpu_api_err!("vulkan queue present {}", e))?,
                    _ => {}
                };
            }
        }

        Ok(())
    }
}
