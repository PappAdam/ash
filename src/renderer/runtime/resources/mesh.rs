use ash::vk;

use super::buffers::{Buffer, BufferAlloc};
use crate::renderer::utilities::Vertex;

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub vertex_count: u64,
    pub index_buffer: Buffer,
    pub index_count: u64,
}

impl Mesh {
    pub fn new(
        vertecies: &[Vertex],
        indicies: &[u16],
        device: &ash::Device,
        buffer_alloc: &BufferAlloc,
    ) -> Self {
        let (vertex_buffer, vertex_count) = Buffer::device_local(
            &vertecies,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            buffer_alloc,
            device,
        );

        let (index_buffer, index_count) = Buffer::device_local(
            &indicies,
            vk::BufferUsageFlags::INDEX_BUFFER,
            buffer_alloc,
            device,
        );

        Self {
            vertex_buffer,
            vertex_count,
            index_buffer,
            index_count,
        }
    }
}
