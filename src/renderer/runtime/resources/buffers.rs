use std::{mem::size_of_val, ptr::copy_nonoverlapping};

use ash::{self, vk};

pub struct BufferAlloc {
    pub command_buffer: vk::CommandBuffer,
    queue: vk::Queue,
    physical_device_mem_props: vk::PhysicalDeviceMemoryProperties,
}

impl BufferAlloc {
    pub fn new(
        physical_device_mem_props: vk::PhysicalDeviceMemoryProperties,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        device: &ash::Device,
    ) -> Self {
        let command_buffer = unsafe {
            device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::builder()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .unwrap()[0]
        };

        Self {
            queue,
            command_buffer,
            physical_device_mem_props,
        }
    }
}

pub struct Buffer {
    pub memory: vk::DeviceMemory,
    pub buffer: vk::Buffer,
}

impl Buffer {
    #[inline]
    pub fn create_buffer(
        buffer_alloc: &BufferAlloc,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        props: vk::MemoryPropertyFlags,
        device: &ash::Device,
    ) -> Self {
        let buffer_info = vk::BufferCreateInfo {
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe {
            device
                .create_buffer(&buffer_info, None)
                .expect("failed to create buffer")
        };

        let mem_reqs = unsafe { device.get_buffer_memory_requirements(buffer) };
        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: mem_reqs.size,
            memory_type_index: Self::find_memory_type(
                mem_reqs.memory_type_bits,
                buffer_alloc.physical_device_mem_props,
                props,
            )
            .expect("No suitable memory type was found"),
            ..Default::default()
        };

        unsafe {
            let memory = device
                .allocate_memory(&alloc_info, None)
                .expect("Failed to allocate memory");

            device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind memory");

            Self { buffer, memory }
        }
    }

    #[inline]
    pub fn device_local<T>(
        instances: &[T],
        usage: vk::BufferUsageFlags,
        buffer_alloc: &BufferAlloc,
        device: &ash::Device,
    ) -> (Self, u64) {
        let size = size_of_val(&instances[0]) as u64 * instances.len() as u64;
        let staging_buffer = Self::create_buffer(
            buffer_alloc,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device,
        );

        let device_local_buffer = Self::create_buffer(
            buffer_alloc,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device,
        );

        unsafe {
            let data = device
                .map_memory(staging_buffer.memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap();

            copy_nonoverlapping(instances.as_ptr(), data as *mut T, size as usize);
            device.unmap_memory(staging_buffer.memory);
        }

        Buffer::copy_to_buffer(
            &[&staging_buffer],
            &[&device_local_buffer],
            &[size],
            buffer_alloc,
            device,
        );

        staging_buffer.free(device);

        (device_local_buffer, instances.len() as u64)
    }

    #[inline]
    pub fn copy_to_buffer(
        src_buffers: &[&Buffer],
        dst_buffer: &[&Buffer],
        sizes: &[u64],
        buffer_alloc: &BufferAlloc,
        device: &ash::Device,
    ) {
        unsafe {
            device
                .begin_command_buffer(
                    buffer_alloc.command_buffer,
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap();

            let mut copy_region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                ..Default::default()
            };

            for i in 0..src_buffers.len() {
                copy_region.size = sizes[i];

                device.cmd_copy_buffer(
                    buffer_alloc.command_buffer,
                    src_buffers[i].buffer,
                    dst_buffer[i].buffer,
                    std::slice::from_ref(&copy_region),
                );
            }

            device
                .end_command_buffer(buffer_alloc.command_buffer)
                .unwrap();

            device
                .queue_submit(
                    buffer_alloc.queue,
                    std::slice::from_ref(
                        &vk::SubmitInfo::builder()
                            .command_buffers(std::slice::from_ref(&buffer_alloc.command_buffer)),
                    ),
                    vk::Fence::null(),
                )
                .unwrap();
            device.queue_wait_idle(buffer_alloc.queue).unwrap();
        }
    }

    fn find_memory_type(
        type_filter: u32,
        mem_props: vk::PhysicalDeviceMemoryProperties,
        props: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        for i in 0..mem_props.memory_type_count {
            if type_filter & (1 << i) != 0
                && mem_props.memory_types[i as usize].property_flags & props == props
            {
                return Some(i);
            }
        }

        None
    }

    #[inline]
    pub fn free(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}
