use crate::engine::lin_alg::{Vector2, Vector3};
use ash::{self, vk};

pub const MAX_FRAME_DRAWS: usize = 3;
pub const MAX_OBJS: usize = 100;

#[derive(Clone, Copy)]
pub struct SwapchainImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
}

impl SwapchainImage {
    pub fn new(image: vk::Image, view: vk::ImageView) -> Self {
        Self { image, view }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub pos: Vector2<f32>,
    pub color: Vector3<f32>,
}

pub struct ViewManipulation {
    pub width_height_ratio: f32,
}

pub struct ObjTransform {
    pub height: f32,
}
