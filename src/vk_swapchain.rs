use ash::vk;
use ash::extensions::khr;

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &khr::Surface,
    ) -> Self {
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to query for surface capabilities.")
        };
        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to query for surface formats.")
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to query for surface present mode.")
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }
}

pub struct Swapchain {
    pub loader: khr::Swapchain,
    pub handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        window_width: u32,
        window_height: u32,
        surface_loader: &khr::Surface,
    ) -> Self {
        let swapchain_support = SwapchainSupportDetails::new(instance, physical_device, surface, surface_loader);

        let surface_format = swapchain_support
            .formats
            .iter()
            .cloned()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_SRGB
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(swapchain_support.formats[0]);

        let present_mode = swapchain_support
            .present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let extent = {
            let mut actual_extent = vk::Extent2D {
                width: window_width,
                height: window_height,
            };

            actual_extent.width = actual_extent
                .width
                .max(swapchain_support.capabilities.min_image_extent.width)
                .min(swapchain_support.capabilities.max_image_extent.width);
            actual_extent.height = actual_extent
                .height
                .max(swapchain_support.capabilities.min_image_extent.height)
                .min(swapchain_support.capabilities.max_image_extent.height);

            actual_extent
        };

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices): (vk::SharingMode, u32, [u32; 0]) =
    (vk::SharingMode::EXCLUSIVE, 0, []);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            //.queue_family_index_count(queue_family_index_count) // This line is not needed and can be removed.
            //.p_queue_family_indices(queue_family_indices) // This line is not needed and can be removed.
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null())
            .build();

        let loader = khr::Swapchain::new(instance, device);
        let handle = unsafe { loader.create_swapchain(&swapchain_create_info, None) }
            .expect("Failed to create Swapchain.");
        let images = unsafe { loader.get_swapchain_images(handle) }.expect("Failed to get Swapchain Images.");

        Self {
            loader,
            handle,
            images,
            image_format: surface_format.format,
            extent,
        }
    }
}
