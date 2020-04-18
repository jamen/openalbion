// Re-export dependencies
pub use winit;
pub use ash;

use std::ffi::{CString,CStr,c_void};
use std::os::raw::c_char;
use std::ptr;
use std::convert::From;

use winit::platform::windows::WindowExtWindows;

use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::extensions::khr::Win32Surface;
use ash::extensions::khr::Swapchain;

pub struct EmberConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

pub struct Ember {
    pub winit_window: winit::window::Window,
    pub winit_event_loop: winit::event_loop::EventLoop<()>,
    pub ash_entry: ash::Entry,
    pub ash_instance: ash::Instance,
    pub ash_physical_device: vk::PhysicalDevice,
    pub ash_queue_family_index: u32,
    pub ash_device: ash::Device,
    pub ash_surface: vk::SurfaceKHR,
    pub ash_surface_handle: Surface,
}

#[derive(Debug)]
pub enum EmberError {
    WinitOsError,
    CStringNulError,
    Utf8Error,
    AshLoadingError(ash::LoadingError),
    AshInstanceError(ash::InstanceError),
    VkFailedResult(vk::Result),
    NoSuitablePhysicalDevice,
    NoSuitableQueueFamily,
    NoLayersAvailable,
    LayerUnsupported,
    DeviceOrSurfaceUnsuitableForSwapchain,
    NoSuitableSurfaceFormat
}

impl From<winit::error::OsError> for EmberError {
    fn from(_: winit::error::OsError) -> Self { EmberError::WinitOsError }
}

impl From<std::ffi::NulError> for EmberError {
    fn from(_: std::ffi::NulError) -> Self { EmberError::CStringNulError }
}

impl From<std::str::Utf8Error> for EmberError {
    fn from(_: std::str::Utf8Error) -> Self { EmberError::Utf8Error }
}

impl From<ash::LoadingError> for EmberError {
    fn from(e: ash::LoadingError) -> Self { EmberError::AshLoadingError(e) }
}

impl From<ash::InstanceError> for EmberError {
    fn from(e: ash::InstanceError) -> Self { EmberError::AshInstanceError(e) }
}

impl From<vk::Result> for EmberError {
    fn from(e: vk::Result) -> Self { EmberError::VkFailedResult(e) }
}

impl Ember {
    pub fn new(config: EmberConfig) -> Result<Self, EmberError> {
        /*
         *
         */

        let window_event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title(&config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height))
            .build(&window_event_loop)?;

        //
        // Create graphics entry
        //

        let ash_entry = ash::Entry::new()?;

        //
        // Create instance
        //

        let app_name = CString::new(config.title.as_str())?;
        let engine_name = CString::new(config.title.as_str())?;

        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_version(1, 0, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_version(1, 0, 0),
            api_version: vk::make_version(1, 0, 92),
        };

        #[allow(unused_mut)]
        let mut required_layer_names: Vec<String> = Vec::new();

        let layer_properties_enumerator = ash_entry.enumerate_instance_layer_properties()?;

        if layer_properties_enumerator.len() <= 0 {
            return Err(EmberError::NoLayersAvailable)
        } else {
            let mut supported_layer_names = Vec::with_capacity(layer_properties_enumerator.len());

            for layer_properties in layer_properties_enumerator {
                let layer_name = unsafe { CStr::from_ptr(layer_properties.layer_name.as_ptr()) };
                let layer_name = layer_name.to_str()?.to_string();

                println!("layer {:?}", &layer_name);

                supported_layer_names.push(layer_name);
            }

            for layer_name in &required_layer_names {
                if !supported_layer_names.contains(&layer_name) {
                    return Err(EmberError::LayerUnsupported)
                }
            }
        }

        let mut layer_names: Vec<*const i8> = Vec::with_capacity(required_layer_names.len());

        for required_layer_name in required_layer_names.iter() {
            layer_names.push(
                CString::new(required_layer_name.as_bytes())?.as_ptr()
            )
        }

        let instance_extension_names = vec![
            DebugUtils::name().as_ptr(),
            Surface::name().as_ptr(),
            Win32Surface::name().as_ptr(),
        ];

        // println!("swapchain extension name {:?}", Swapchain::name().to_str()?.to_string());

        // let instance_extension_properties_enumerator = ash_entry.enumerate_instance_extension_properties()?;

        // for &instance_extension_properties in instance_extension_properties_enumerator.iter() {
        //     let extension_name = unsafe { CStr::from_ptr(instance_extension_properties.extension_name.as_ptr()) };
        //     let extension_name = extension_name.to_str()?.to_string();
        //     println!("instance extension {:?}", &extension_name);
        // }

        let instance_create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: layer_names.as_ptr(),
            enabled_layer_count: layer_names.len() as u32,
            pp_enabled_extension_names: instance_extension_names.as_ptr(),
            enabled_extension_count: instance_extension_names.len() as u32,
        };

        let ash_instance = unsafe { ash_entry.create_instance(&instance_create_info, None)? };

        //
        // Validation layer
        //

        // #[cfg(feature = "debug")]
        // {
        //     let debug_utils_loader = DebugUtils::new(&ash_entry, &ash_instance);

        //     let debug_utils_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
        //         s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        //         p_next: ptr::null(),
        //         flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        //         message_severity:
        //             vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
        //             vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        //         message_type:
        //             vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
        //             vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
        //             vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        //             pfn_user_callback: Some(Self::debug_utils_callback),
        //             p_user_data: ptr::null_mut(),
        //     };

        //     unsafe {
        //         debug_utils_loader.create_debug_utils_messenger(&debug_utils_messenger_create_info, None)?;
        //     }
        // }

        //
        // Select physical device and queue family
        //
        // TODO: Select the best device by ranking them all.
        // TODO: Use vk::PhysicalDeviceType::DISCRETE_GPU to rank a graphics card higher than the integrated GPU.

        let mut physical_device: Option<vk::PhysicalDevice> = None;
        let mut queue_family_index: Option<u32> = None;

        let physical_devices_enumerator = unsafe { ash_instance.enumerate_physical_devices()? };

        'select_physical_device: for &next_physical_device in physical_devices_enumerator.iter() {
            // let device_properties = unsafe { instance.get_physical_device_properties(next_physical_device) };
            let device_features  = unsafe { ash_instance.get_physical_device_features(next_physical_device) };

            let device_queue_families =
                unsafe { ash_instance.get_physical_device_queue_family_properties(next_physical_device) };

            if
                // TODO: Supposedly MoltenVK on MacOS does not support geometry shaders. Find workarounds.
                device_features.geometry_shader == 1
            {
                let mut next_queue_family_index: u32 = 0;

                for next_queue_family in device_queue_families.iter() {
                    if
                        next_queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && next_queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                        && next_queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                        && next_queue_family.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING)
                    {
                        physical_device = Some(next_physical_device);
                        queue_family_index = Some(next_queue_family_index);
                        break 'select_physical_device;
                    }

                    next_queue_family_index += 1;
                }
            }
        }

        let ash_physical_device = match physical_device {
            Some(physical_device) => physical_device,
            None => return Err(EmberError::NoSuitablePhysicalDevice),
        };

        let ash_queue_family_index = match queue_family_index {
            Some(queue_family_index) => queue_family_index,
            None => return Err(EmberError::NoSuitableQueueFamily)
        };

        let device_extension_properties_enumerator = unsafe {
            ash_instance.enumerate_device_extension_properties(ash_physical_device)?
        };

        for &extension_properties in device_extension_properties_enumerator.iter() {
            let extension_name = unsafe { CStr::from_ptr(extension_properties.extension_name.as_ptr()) };
            let extension_name = extension_name.to_str()?.to_string();
            println!("device extension {:?}", &extension_name);
        }

        //
        // Create logical device
        //

        let device_queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: ash_queue_family_index,
            queue_count: 1,
            p_queue_priorities: [1.0_f32].as_ptr()
        };

        let device_extension_names = vec![
            Swapchain::name().as_ptr(),
        ];

        let physical_device_features: vk::PhysicalDeviceFeatures = Default::default();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &device_queue_create_info,
            enabled_layer_count: layer_names.len() as u32,
            pp_enabled_layer_names: layer_names.as_ptr(),
            pp_enabled_extension_names: device_extension_names.as_ptr(),
            enabled_extension_count: device_extension_names.len() as u32,
            p_enabled_features: &physical_device_features,
        };

        let ash_device = unsafe {
            ash_instance.create_device(ash_physical_device, &device_create_info, None)?
        };

        // TODO: Double check availability of swapchain

        //
        // Window surface
        //
        // TODO: Fix this for other platforms than windows.

        let hwnd = window.hwnd();
        let hinstance = unsafe { winapi::um::libloaderapi::GetModuleHandleW(ptr::null()) };

        let win32_create_surface_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: Default::default(),
            hinstance: hinstance as *const c_void,
            hwnd: hwnd,
        };

        let win32_surface_loader = Win32Surface::new(&ash_entry, &ash_instance);

        let ash_surface = unsafe { win32_surface_loader.create_win32_surface(&win32_create_surface_info, None)? };

        let ash_surface_handle = Surface::new(&ash_entry, &ash_instance);

        //
        // Check surface capabilities for swapchain.
        //

        let ash_surface_handle_capabilities = unsafe {
            ash_surface_handle.get_physical_device_surface_capabilities(ash_physical_device, ash_surface)?
        };

        let ash_surface_handle_formats = unsafe {
            ash_surface_handle.get_physical_device_surface_formats(ash_physical_device, ash_surface)?
        };

        let ash_surface_handle_present_modes = unsafe {
            ash_surface_handle.get_physical_device_surface_present_modes(ash_physical_device, ash_surface)?
        };

        if ash_surface_handle_formats.is_empty() || ash_surface_handle_present_modes.is_empty() {
            return Err(EmberError::DeviceOrSurfaceUnsuitableForSwapchain)
        }

        //
        // Select swapchain format, extent, and present.
        //

        let mut image_format: Option<vk::SurfaceFormatKHR> = None;

        for &candidate in ash_surface_handle_formats.iter() {
            if
                candidate.format == vk::Format::B8G8R8A8_SRGB &&
                candidate.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                image_format = Some(candidate.clone());
            }
        }

        let image_format = match image_format {
            Some(f) => f,
            None => match ash_surface_handle_formats.get(0) {
                Some(f) => f.clone(),
                None => {
                    return Err(EmberError::NoSuitableSurfaceFormat)
                }
            },
        };


        let mut present_mode: Option<vk::PresentModeKHR> = None;

        for &candidate in ash_surface_handle_present_modes.iter() {
            if candidate == vk::PresentModeKHR::MAILBOX {
                present_mode = Some(candidate.clone());
            }
        }

        let present_mode = match present_mode {
            Some(p) => p,
            None => vk::PresentModeKHR::FIFO,
        };


        let image_extent: vk::Extent2D = vk::Extent2D {
            width: ash_surface_handle_capabilities.min_image_extent.width.max(ash_surface_handle_capabilities.max_image_extent.width.max(config.width)),
            height: ash_surface_handle_capabilities.min_image_extent.height.max(ash_surface_handle_capabilities.max_image_extent.height.max(config.height)),
        };

        //
        // Create swapchain
        //

        let image_count = if ash_surface_handle_capabilities.max_image_count > 0 {
            ash_surface_handle_capabilities.min_image_count + 1
        } else {
            ash_surface_handle_capabilities.max_image_count
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: ash_surface,
            min_image_count: image_count,
            image_format: image_format.format,
            image_color_space: image_format.color_space,
            image_extent: image_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            pre_transform: ash_surface_handle_capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
        };

        let swapchain = Swapchain::new(&ash_instance, &ash_device);

        let swapchain = unsafe {
            swapchain.create_swapchain(&swapchain_create_info, None)?
        };

        //
        // TODO...
        //

        Ok(
            Ember {
                winit_window: window,
                winit_event_loop: window_event_loop,
                ash_entry: ash_entry,
                ash_instance: ash_instance,
                ash_physical_device: ash_physical_device,
                ash_queue_family_index: ash_queue_family_index,
                ash_device: ash_device,
                ash_surface: ash_surface,
                ash_surface_handle: ash_surface_handle,
            }
        )
    }

    // // Not actually dead code..?
    // //
    // // TODO: Probably change this.
    // #[allow(dead_code)]
    // unsafe extern "system" fn debug_utils_callback(
    //     message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    //     message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    //     p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    //     _p_user_data: *mut std::ffi::c_void,
    // ) -> vk::Bool32 {
    //     let severity = match message_severity {
    //         vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
    //         vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
    //         vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
    //         vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
    //         _ => "[Unknown]",
    //     };

    //     let types = match message_type {
    //         vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
    //         vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
    //         vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
    //         _ => "[Unknown]",
    //     };

    //     let message = CStr::from_ptr((*p_callback_data).p_message);

    //     println!("[Debug]{}{}{:?}", severity, types, message);

    //     vk::FALSE
    // }
}