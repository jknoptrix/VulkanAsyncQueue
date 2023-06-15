use crate::vk_swapchain::Swapchain;

use ash::{
    vk,
    extensions::khr
};



pub struct FrameManager {
    device: ash::Device,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    present_mode: vk::PresentModeKHR,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    swapchain_module: std::rc::Rc<Swapchain>,
}

impl FrameManager {
    pub fn new(
        device: ash::Device,
        swapchain_loader: ash::extensions::khr::Swapchain,
        swapchain: vk::SwapchainKHR,
        present_mode: vk::PresentModeKHR,
        surface_format: vk::SurfaceFormatKHR,
        extent: vk::Extent2D,
        swapchain_module: std::rc::Rc<Swapchain>,

    ) -> Self {
        Self {
            device,
            swapchain_loader,
            swapchain,
            present_mode,
            surface_format,
            extent,
            swapchain_module,
        }
    }

    pub fn recreate_swapchain(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        window_width: u32,
        window_height: u32,
        surface_loader: &khr::Surface,
    ) {
        unsafe {
            device.device_wait_idle().unwrap();
        }

        self.swapchain_module = std::rc::Rc::new(Swapchain::new(
            instance,
            device,
            physical_device,
            surface,
            window_width,
            window_height,
            surface_loader,
        ));

        self.swapchain = self.swapchain_module.handle;
        self.extent = self.swapchain_module.extent;
    }

    pub fn acquire_next_image(
        &self,
        image_available_semaphore: vk::Semaphore,
    ) -> Result<(u32, bool), vk::Result> {
        let (image_index, suboptimal) = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            )?
        };

        Ok((image_index, suboptimal))
    }

    pub fn present_image(
        &self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool, vk::Result> {
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(&[self.swapchain])
            .image_indices(&[image_index])
            .build();

        let result = unsafe { self.swapchain_loader.queue_present(queue, &present_info) };

        match result {
            Ok(is_suboptimal) => Ok(is_suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok(true),
            Err(error) => Err(error),
        }
    }
}
