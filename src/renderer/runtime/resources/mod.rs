use std::alloc::Layout;

use ash::vk;

use crate::renderer::utilities::ObjTransform;

use self::{buffers::Buffer, mesh::Mesh};

use super::Renderer;

pub mod buffers;
pub mod mesh;

pub struct Resources {
    meshes: Vec<Mesh>,

    // Descriptors
    view_buffers: Vec<Buffer>,
    obj_transfrom_buffers: Vec<Buffer>,

    // system infos
    uniform_buffer_alignment: usize,
    minimum_uniform_buffer_offset: u64,
    obj_transform_allocation_layout: Layout,
    obj_transform_transfer_space_memory: *mut ObjTransform,
}

impl Resources {
    fn new(renderer: &Renderer) -> Self {
        Self {
            meshes: (),
            view_buffers: (),
            obj_transfrom_buffers: (),
            uniform_buffer_alignment: (),
            minimum_uniform_buffer_offset: (),
            obj_transform_allocation_layout: (),
            obj_transform_transfer_space_memory: (),
        }
    }
}
