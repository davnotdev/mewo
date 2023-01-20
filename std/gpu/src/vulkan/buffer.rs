use super::*;
use std::mem::ManuallyDrop;

//  TODO OPT: Question on optimizing buffers. So right now, pretty much everything relies on
//  staging buffer. Great right? GpuOnly == Better Performance, but wait. What about uploads?
//  staging buffers require a map, then upload. For per frame uploads, shouldn't it just be better
//  to use the Gpu Cpu shared buffers? I'm not sure if the CpuToGpu option allows for this, so for
//  now, staging buffers are the norm.

impl VkContext {
    pub fn new_vertex_buffer(
        &mut self,
        data: &[GpuVertexBufferElement],
        storage_type: GpuBufferStorageType,
    ) -> GResult<GpuVertexBufferId> {
        let (buf, staging) =
            self.new_generic_buffer(data, storage_type, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let vbo = VkVertexBuffer {
            buffer: buf,
            staging,
        };
        self.vbos.push(vbo);
        Ok(GpuVertexBufferId::from_id(self.vbos.len() - 1))
    }

    pub fn new_index_buffer(
        &mut self,
        data: &[GpuIndexBufferElement],
        storage_type: GpuBufferStorageType,
    ) -> GResult<GpuIndexBufferId> {
        let (buf, staging) =
            self.new_generic_buffer(data, storage_type, vk::BufferUsageFlags::INDEX_BUFFER)?;
        let ibo = VkIndexBuffer {
            buffer: buf,
            staging,
        };
        self.ibos.push(ibo);
        Ok(GpuIndexBufferId::from_id(self.ibos.len() - 1))
    }
}

pub struct VkVertexBuffer {
    pub buffer: VkBuffer,
    staging: Option<VkBuffer>,
}

pub struct VkIndexBuffer {
    pub buffer: VkBuffer,
    staging: Option<VkBuffer>,
}

impl VkContext {
    fn new_generic_buffer<T>(
        &mut self,
        data: &[T],
        storage_type: GpuBufferStorageType,
        additional_buffer_usage: vk::BufferUsageFlags,
    ) -> GResult<(VkBuffer, Option<VkBuffer>)> {
        let buf_size = std::mem::size_of::<T>() * data.len();

        match storage_type {
            GpuBufferStorageType::Static => {}
            GpuBufferStorageType::Dynamic => {}
        };
        let staging = VkBuffer::new(
            &self.core.dev,
            &self.drop_queue,
            &mut self.alloc,
            buf_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;
        staging.map_copy_data(data.as_ptr() as *const u8, buf_size)?;
        let buf = VkBuffer::new(
            &self.core.dev,
            &self.drop_queue,
            &mut self.alloc,
            buf_size,
            vk::BufferUsageFlags::TRANSFER_DST | additional_buffer_usage,
            MemoryLocation::GpuOnly,
        )?;
        {
            let fence = new_fence(&self.core.dev, false)?;
            let transfer_command_buf_alloc = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.core.compute_command_pool)
                .command_buffer_count(1)
                .build();
            let transfer_command_buf = unsafe {
                self.core
                    .dev
                    .allocate_command_buffers(&transfer_command_buf_alloc)
            }
            .map_err(|e| gpu_api_err!("vulkan buffer alloc command buffer {}", e))?
            .into_iter()
            .next()
            .unwrap();
            VkBuffer::single_upload_copy_data(
                &staging,
                &buf,
                &self.core.dev,
                self.core.graphics_queue,
                fence,
                transfer_command_buf,
            )?;
            unsafe { self.core.dev.destroy_fence(fence, None) }
        }

        Ok((
            buf,
            if storage_type == GpuBufferStorageType::Dynamic {
                Some(staging)
            } else {
                None
            },
        ))
    }
}

pub struct VkBuffer {
    pub buffer: vk::Buffer,
    pub allocation: ManuallyDrop<Allocation>,
    pub size: usize,

    drop_queue_ref: VkDropQueueRef,
}

impl VkBuffer {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        data_size: usize,
        usage: vk::BufferUsageFlags,
        mem_usage: MemoryLocation,
    ) -> GResult<Self> {
        //  Create and Allocate the Buffer
        let buffer_create = vk::BufferCreateInfo::builder()
            .size(data_size as u64)
            .usage(usage)
            .build();

        let buffer = unsafe { dev.create_buffer(&buffer_create, None) }.unwrap();
        let requirements = unsafe { dev.get_buffer_memory_requirements(buffer) };

        let allocation = alloc
            .allocate(&AllocationCreateDesc {
                //  TODO EXT: Support the naming of buffers.
                name: "Vulkan Generic Buffer",
                requirements,
                location: mem_usage,
                linear: true,
            })
            .unwrap();

        unsafe {
            dev.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .map_err(|e| gpu_api_err!("vulkan buffer bind {}", e))?;
        };

        Ok(VkBuffer {
            buffer,
            allocation: ManuallyDrop::new(allocation),
            size: data_size,

            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }

    pub fn map_copy_data(&self, ptr: *const u8, size: usize) -> GResult<()> {
        //  Fill the Buffer
        unsafe {
            let data = self
                .allocation
                .mapped_ptr()
                .ok_or(gpu_api_err!(
                    "vulkan gpu_allocator, this buffer cannot be mapped"
                ))?
                .as_ptr();
            std::ptr::copy_nonoverlapping::<u8>(ptr, data as *mut u8, size);
        };
        Ok(())
    }

    pub fn cmd_upload_copy_data(
        src: &Self,
        dst: &Self,
        dev: &Device,
        command_buf: vk::CommandBuffer,
    ) {
        let copy_create = vk::BufferCopy::builder().size(src.size as u64).build();
        unsafe { dev.cmd_copy_buffer(command_buf, src.buffer, dst.buffer, &[copy_create]) };
    }

    pub fn single_upload_copy_data(
        src: &Self,
        dst: &Self,
        dev: &Device,
        transfer_queue: vk::Queue,
        fence: vk::Fence,
        command_buf: vk::CommandBuffer,
    ) -> GResult<()> {
        if src.size > dst.size {
            Err(gpu_api_err!(
                "vulkan upload copy src.size ({}) > dst.size ({})",
                src.size,
                dst.size
            ))?
        }
        unsafe {
            let cmd_begin_create = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();
            dev.begin_command_buffer(command_buf, &cmd_begin_create)
                .map_err(|e| gpu_api_err!("vulkan upload copy begin command buffer {}", e))?;

            Self::cmd_upload_copy_data(src, dst, dev, command_buf);

            dev.end_command_buffer(command_buf)
                .map_err(|e| gpu_api_err!("vulkan upload copy end command buffer {}", e))?;

            let submit_create = vk::SubmitInfo::builder()
                .command_buffers(&[command_buf])
                .build();
            dev.queue_submit(transfer_queue, &[submit_create], fence)
                .map_err(|e| gpu_api_err!("vulkan upload copy submit {}", e))?;
            dev.wait_for_fences(&[fence], true, std::u64::MAX)
                .map_err(|e| gpu_api_err!("vulkan upload copy wait fence {}", e))?;

            //  Cleanup
            dev.reset_fences(&[fence])
                .map_err(|e| gpu_api_err!("vulkan upload copy reset fence {}", e))?;
            dev.reset_command_buffer(command_buf, vk::CommandBufferResetFlags::empty())
                .map_err(|e| gpu_api_err!("vulkan upload copy reset command buffer {}", e))?;
        }
        Ok(())
    }
}

//  TODO FIX: Drop Staging Buffer.
impl Drop for VkBuffer {
    fn drop(&mut self) {
        let buffer = self.buffer;
        let allocation = unsafe { ManuallyDrop::take(&mut self.allocation) };
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, alloc| unsafe {
                alloc.free(allocation).unwrap();
                dev.destroy_buffer(buffer, None);
            }));
    }
}
