use ash::{
    extensions::khr::{Surface, Swapchain},
    util::read_spv,
    vk,
};
use std::io::Cursor;
use std::{ffi::CStr, mem::size_of};

use winit::window::Window;

use crate::offset_of;

use super::utilities::{SwapchainImage, Vertex, MAX_FRAME_DRAWS};

pub fn create_descriptor_pool(
    device: &ash::Device,
    pool_sizes: &[vk::DescriptorPoolSize],
    swapchain_imgs_len: u32,
) -> vk::DescriptorPool {
    let pool_create_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(swapchain_imgs_len);

    unsafe {
        device
            .create_descriptor_pool(&pool_create_info, None)
            .expect("Failed to create descriptor pool")
    }
}

pub fn create_descriptor_set_layout(
    device: &ash::Device,
    layout_bindings: &[vk::DescriptorSetLayoutBinding],
) -> vk::DescriptorSetLayout {
    let layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(layout_bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_create_info, None)
            .expect("Failed to create descriptor set layout")
    }
}

pub fn create_command_buffers(
    device: &ash::Device,
    command_pool: &vk::CommandPool,
) -> Vec<vk::CommandBuffer> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(*command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_FRAME_DRAWS as u32);

    unsafe {
        device
            .allocate_command_buffers(&alloc_info)
            .expect("Failed to create command buffer(s)")
    }
}

pub fn create_frame_buffers(
    swapchain_imgs: &Vec<SwapchainImage>,
    render_pass: &vk::RenderPass,
    extent: &vk::Extent2D,
    device: &ash::Device,
) -> Vec<vk::Framebuffer> {
    swapchain_imgs
        .iter()
        .map(|img| {
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass)
                .attachments(std::slice::from_ref(&img.view))
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create framebuffer")
            }
        })
        .collect::<Vec<vk::Framebuffer>>()
}

pub fn create_pipeline(
    device: &ash::Device,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
    extent: &vk::Extent2D,
    render_pass: &vk::RenderPass,
) -> (vk::Pipeline, vk::PipelineLayout, vk::Viewport, vk::Rect2D) {
    let mut vertex_spv = Cursor::new(&include_bytes!("../complied_shaders/vert.spv")[..]);
    let mut frag_spv = Cursor::new(&include_bytes!("../complied_shaders/frag.spv")[..]);

    let vertex_code = read_spv(&mut vertex_spv).expect("Failed to read vertex shader spv");
    let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
    let frag_code = read_spv(&mut frag_spv).expect("Failed to read fragment shader spv");
    let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);

    let vertex_module = unsafe {
        device
            .create_shader_module(&vertex_shader_info, None)
            .unwrap()
    };

    let frag_module = unsafe {
        device
            .create_shader_module(&frag_shader_info, None)
            .unwrap()
    };

    let layout_create_info =
        vk::PipelineLayoutCreateInfo::builder().set_layouts(descriptor_set_layouts);
    let pipeline_layout = unsafe {
        device
            .create_pipeline_layout(&layout_create_info, None)
            .unwrap()
    };

    let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo {
            module: vertex_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            module: frag_module,
            p_name: shader_entry_name.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        },
    ];

    let vertex_bind_desc = vk::VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };

    let attribute_desc = [
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: unsafe { offset_of!(Vertex, pos) } as u32,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: unsafe { offset_of!(Vertex, color) } as u32,
        },
    ];

    let vertex_input_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(std::slice::from_ref(&vertex_bind_desc))
        .vertex_attribute_descriptions(&attribute_desc);

    // let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    // let dynamic_state_create_info =
    //     vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

    let viewports = [vk::Viewport {
        x: 0f32,
        y: 0f32,
        width: extent.width as f32,
        height: extent.height as f32,
        min_depth: 0f32,
        max_depth: 1f32,
    }];

    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: *extent,
    }];

    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewports)
        .scissors(&scissors);

    let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };

    let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::CLOCKWISE,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        line_width: 1.0,
        ..Default::default()
    };

    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };

    let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&color_blend_attachment_states);

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_infos)
        .viewport_state(&viewport_state_create_info)
        .vertex_input_state(&vertex_input_create_info)
        .input_assembly_state(&input_assembly_create_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(*render_pass)
        // .dynamic_state(&dynamic_state_create_info)
        ;

    unsafe {
        let pipelines = device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_create_info),
                None,
            )
            .unwrap();

        device.destroy_shader_module(vertex_module, None);
        device.destroy_shader_module(frag_module, None);

        (pipelines[0], pipeline_layout, viewports[0], scissors[0])
    }
}

pub fn create_render_pass(format: vk::Format, device: &ash::Device) -> vk::RenderPass {
    let rendepass_attachments = [vk::AttachmentDescription {
        format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    }];

    let color_attachment_refs = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];

    let dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ..Default::default()
    }];

    let subpass = vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_refs)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&rendepass_attachments)
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(&dependencies);

    unsafe {
        device
            .create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass")
    }
}

pub fn create_semaphores(device: &ash::Device) -> Vec<vk::Semaphore> {
    let mut semaphore_vec: Vec<vk::Semaphore> = Vec::with_capacity(MAX_FRAME_DRAWS);
    unsafe {
        for _ in 0..MAX_FRAME_DRAWS {
            semaphore_vec.push(
                device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .expect("Failed to create semaphore"),
            )
        }
    };
    semaphore_vec
}

pub fn create_signalled_fences(device: &ash::Device) -> Vec<vk::Fence> {
    let mut fence_vec: Vec<vk::Fence> = Vec::with_capacity(MAX_FRAME_DRAWS);
    unsafe {
        for _ in 0..MAX_FRAME_DRAWS {
            fence_vec.push(
                device
                    .create_fence(
                        &vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .expect("Failed to create fence"),
            )
        }
    };
    fence_vec
}

pub fn create_swapchain_images(
    swapchain_loader: &Swapchain,
    swapchain: &vk::SwapchainKHR,
    device: &ash::Device,
    format: vk::Format,
) -> Vec<SwapchainImage> {
    unsafe {
        swapchain_loader
            .get_swapchain_images(*swapchain)
            .unwrap()
            .into_iter()
            .map(|img| {
                let view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(img);

                let view = device.create_image_view(&view_info, None).unwrap();

                SwapchainImage::new(img, view)
            })
            .collect::<Vec<_>>()
    }
}

pub fn create_swapchain(
    swapchain_loader: &Swapchain,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
    physical_device: &vk::PhysicalDevice,
    window: &Window,
) -> (vk::SwapchainKHR, vk::SurfaceFormatKHR, vk::Extent2D) {
    let surface_format = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*physical_device, *surface)
            .unwrap()
            .into_iter()
            .min_by_key(|f| match (f.format, f.color_space) {
                (vk::Format::R8G8B8_UNORM, vk::ColorSpaceKHR::SRGB_NONLINEAR) => 0,
                (vk::Format::B8G8R8_UNORM, vk::ColorSpaceKHR::SRGB_NONLINEAR) => 1,
                _ => 2,
            })
            .expect("No format is supported")
    };

    let surface_caps = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*physical_device, *surface)
            .unwrap()
    };

    let image_count = {
        let count = surface_caps.min_image_count + 1;

        if surface_caps.max_image_count > 0 && count > surface_caps.max_image_count {
            surface_caps.max_image_count
        } else {
            count
        }
    };

    let extent = match surface_caps.current_extent.width {
        std::u32::MAX => vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        },
        _ => surface_caps.current_extent,
    };

    let present_mode = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*physical_device, *surface)
            .unwrap()
            .into_iter()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(*surface)
        .min_image_count(image_count)
        .image_color_space(surface_format.color_space)
        .image_format(surface_format.format)
        .image_extent(extent)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(surface_caps.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .image_array_layers(1);

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create swapchain")
    };

    (swapchain, surface_format, extent)
}

pub fn create_logical_device(
    instance: &ash::Instance,
    queue_family_index: u32,
    physical_device: &vk::PhysicalDevice,
) -> (ash::Device, vk::Queue) {
    let device_extensions_raw = [Swapchain::name().as_ptr()];

    let features = vk::PhysicalDeviceFeatures::builder();
    let priorities = [1f32];

    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities);

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(std::slice::from_ref(&queue_create_info))
        .enabled_extension_names(&device_extensions_raw)
        .enabled_features(&features);

    let device = unsafe {
        instance
            .create_device(*physical_device, &device_create_info, None)
            .expect("Failed to create device")
    };

    let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    (device, queue)
}

// ========================= GET FUNCTIONS =================================
//
pub fn get_physical_device(
    instance: &ash::Instance,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
) -> (vk::PhysicalDevice, u32) {
    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

    physical_devices
        .iter()
        .filter_map(|p| unsafe {
            instance
                .get_physical_device_queue_family_properties(*p)
                .iter()
                .enumerate()
                .find_map(|(i, info)| {
                    if info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && surface_loader
                            .get_physical_device_surface_support(*p, i as u32, *surface)
                            .unwrap()
                    {
                        Some((*p, i as u32))
                    } else {
                        None
                    }
                })
        })
        .min_by_key(|(p, _)| {
            match unsafe { instance.get_physical_device_properties(*p).device_type } {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                vk::PhysicalDeviceType::CPU => 2,
                vk::PhysicalDeviceType::VIRTUAL_GPU => 3,
                vk::PhysicalDeviceType::OTHER => 4,
                _ => 5,
            }
        })
        .expect("Failed to find proper physical device")
}
