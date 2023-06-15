use ash::vk;
use log::*;

use std::ffi::CStr;


pub struct DebugUtils {
    loader: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugUtils {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Self {
        let loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        let messenger_ci = vk::DebugUtilsMessengerCreateInfoEXT::builder()
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
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build();

        let messenger = unsafe {
            loader
                .create_debug_utils_messenger(&messenger_ci, None)
                .expect("Failed to create debug utils messenger")
        };

        Self { loader, messenger }
    }
}

impl Drop for DebugUtils {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;

    let message_severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::Level::Trace,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::Level::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::Level::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::Level::Error,
        _ => log::Level::Error,
    };

    let message = CStr::from_ptr(callback_data.p_message).to_string_lossy();
    log!(message_severity, "{}", message);

    vk::FALSE
}
