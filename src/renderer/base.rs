use std::{
    borrow::Cow,
    ffi::{c_char, CStr},
};

use ash::{
    self,
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk::{self, Semaphore},
};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use winit::window::Window;

use super::{
    runtime::resources::buffers::{Buffer, BufferAlloc},
    setup,
    utilities::{SwapchainImage, MAX_FRAME_DRAWS},
};

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity, message_type, message_id_name, message_id_number, message,
    );

    vk::FALSE
}

pub struct RendererBase<'a> {
    pub instance: ash::Instance,
    pub window: &'a Window,

    pub surface: vk::SurfaceKHR,
    pub surface_loader: Surface,
    pub surface_extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,

    // ================= DEBUG ===============
    pub debug_call_back: vk::DebugUtilsMessengerEXT,
    pub debug_utils_loader: DebugUtils,

    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue: vk::Queue,

    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: Swapchain,
    pub swapchain_imgs: Vec<SwapchainImage>,

    pub command_buffers: Vec<vk::CommandBuffer>,
    pub command_pool: vk::CommandPool,

    pub buffer_alloc: BufferAlloc,

    pub img_available: Vec<vk::Semaphore>,
    pub render_finished: Vec<vk::Semaphore>,
    pub next_frame: Vec<vk::Fence>,

    pub current_frame: usize,
}

impl<'a> RendererBase<'a> {
    pub fn new(window: &'a Window) -> Self {
        let entry = ash::Entry::linked();
        let layer_names = [b"VK_LAYER_KHRONOS_validation\0"];

        let layer_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr() as *const _ as *const c_char)
            .collect();

        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .unwrap()
                .to_vec();
        extension_names.push(DebugUtils::name().as_ptr());
        extension_names.push(b"VK_KHR_portability_enumeration\0" as *const _ as *const c_char);

        let app_info = vk::ApplicationInfo::builder()
            .application_version(0)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        // ================== ~~DEBUG~~ ===========================================

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        let debug_call_back = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        // ================== !!DEBUG!! ===========================================

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .unwrap()
        };

        let surface_loader = Surface::new(&entry, &instance);

        let (physical_device, queue_family_index) =
            setup::get_physical_device(&instance, &surface_loader, &surface);

        let (device, queue) =
            setup::create_logical_device(&instance, queue_family_index, &physical_device);

        let swapchain_loader = Swapchain::new(&instance, &device);
        let (swapchain, surface_format, surface_extent) = setup::create_swapchain(
            &swapchain_loader,
            &surface_loader,
            &surface,
            &physical_device,
            window,
        );

        let swapchain_imgs = setup::create_swapchain_images(
            &swapchain_loader,
            &swapchain,
            &device,
            surface_format.format,
        );

        let command_pool_create_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index,
            ..Default::default()
        };
        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create command pool")
        };

        let command_buffers = setup::create_command_buffers(&device, &command_pool);

        let physical_device_mem_props =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let buffer_alloc =
            BufferAlloc::new(physical_device_mem_props, command_pool, queue, &device);

        let img_available = setup::create_semaphores(&device);
        let render_finished = setup::create_semaphores(&device);
        let next_frame = setup::create_signalled_fences(&device);

        Self {
            instance,
            window,
            surface,
            surface_loader,
            surface_extent,
            surface_format,
            debug_call_back,
            debug_utils_loader,
            physical_device,
            device,
            queue,
            swapchain,
            swapchain_loader,
            swapchain_imgs,
            command_buffers,
            command_pool,
            buffer_alloc,
            img_available,
            render_finished,
            next_frame,
            current_frame: 0,
        }
    }
}
