use ash::{extensions::khr::Swapchain, vk};
use winit::window::Window;

use super::{base::RendererBase, setup, utilities::MAX_FRAME_DRAWS};

pub mod resources;
pub mod run;

pub struct Renderer<'a> {
    base: RendererBase<'a>,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    viewport: vk::Viewport,
    scissors: vk::Rect2D,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl<'a> Renderer<'a> {
    pub fn new(window: &'a Window) -> Self {
        let base = RendererBase::new(window);
        let render_pass = setup::create_render_pass(base.surface_format.format, &base.device);

        let descriptor_set_layout_bindings = [
            // View descriptor set
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            // Object transform descriptor set
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
        ];

        let descriptor_pool_sizes = [
            // view descriptor size
            vk::DescriptorPoolSize::builder()
                .descriptor_count(base.swapchain_imgs.len() as u32)
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .build(),
            // obj transform descriptor size
            vk::DescriptorPoolSize::builder()
                .descriptor_count(base.swapchain_imgs.len() as u32)
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .build(),
        ];

        let descriptor_pool = setup::create_descriptor_pool(
            &base.device,
            &descriptor_pool_sizes,
            base.swapchain_imgs.len() as u32,
        );
        let descriptor_set_layout =
            setup::create_descriptor_set_layout(&base.device, &descriptor_set_layout_bindings);

        let (pipeline, pipeline_layout, viewport, scissors) = setup::create_pipeline(
            &base.device,
            std::slice::from_ref(&descriptor_set_layout),
            &base.surface_extent,
            &render_pass,
        );

        let framebuffers = setup::create_frame_buffers(
            &base.swapchain_imgs,
            &render_pass,
            &base.surface_extent,
            &base.device,
        );

        Self {
            base,
            render_pass,
            descriptor_set_layout,
            framebuffers,
            pipeline,
            pipeline_layout,
            viewport,
            scissors,
            descriptor_pool,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        unsafe {
            self.base.device.device_wait_idle().unwrap();
            self.cleanup_swapchain();

            let swapchain_loader = Swapchain::new(&self.base.instance, &self.base.device);
            let (swapchain, format, extent) = setup::create_swapchain(
                &swapchain_loader,
                &self.base.surface_loader,
                &self.base.surface,
                &self.base.physical_device,
                self.base.window,
            );

            let swapchain_imgs = setup::create_swapchain_images(
                &swapchain_loader,
                &swapchain,
                &self.base.device,
                format.format,
            );

            let framebuffers = setup::create_frame_buffers(
                &swapchain_imgs,
                &self.render_pass,
                &extent,
                &self.base.device,
            );

            self.base.swapchain_loader = swapchain_loader;
            self.base.swapchain = swapchain;
            self.base.swapchain_imgs = swapchain_imgs;
            self.framebuffers = framebuffers;
        }
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.framebuffers.iter().for_each(|&fb| {
                self.base.device.destroy_framebuffer(fb, None);
            });
            self.base.swapchain_imgs.iter().for_each(|&img| {
                self.base.device.destroy_image_view(img.view, None);
            });
            self.base
                .swapchain_loader
                .destroy_swapchain(self.base.swapchain, None);
        }
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.base.device.device_wait_idle().unwrap();

            self.base
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.base
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            for i in 0..MAX_FRAME_DRAWS {
                self.base
                    .device
                    .destroy_fence(self.base.next_frame[i], None);
                self.base
                    .device
                    .destroy_semaphore(self.base.img_available[i], None);
                self.base
                    .device
                    .destroy_semaphore(self.base.render_finished[i], None);
            }

            self.base.device.destroy_render_pass(self.render_pass, None);
            self.base
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.base.device.destroy_pipeline(self.pipeline, None);

            self.base
                .device
                .destroy_command_pool(self.base.command_pool, None);

            self.cleanup_swapchain();

            self.base
                .surface_loader
                .destroy_surface(self.base.surface, None);

            self.base.device.destroy_device(None);
            self.base
                .debug_utils_loader
                .destroy_debug_utils_messenger(self.base.debug_call_back, None);
            self.base.instance.destroy_instance(None);
        }
    }
}
