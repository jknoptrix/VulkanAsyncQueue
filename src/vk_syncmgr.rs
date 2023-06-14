use ash::vk;

pub struct SynchronizationManager {
    device: ash::Device,
}

impl SynchronizationManager {
    pub fn new(device: ash::Device) -> Self {
        Self { device }
    }

    pub fn create_semaphore(&self) -> vk::Semaphore {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        unsafe { self.device.create_semaphore(&semaphore_info, None).unwrap() }
    }

    pub fn create_fence(&self) -> vk::Fence {
        let fence_info = vk::FenceCreateInfo::default();
        unsafe { self.device.create_fence(&fence_info, None).unwrap() }
    }
}
