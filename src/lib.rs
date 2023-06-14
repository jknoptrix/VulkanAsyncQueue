pub mod vk_swapchain;
pub mod vk_resmgr;
pub mod vk_syncmgr;
pub mod vk_deskmgr;
pub mod vk_renderpassmgr;
pub mod vk_subpassmgr;
pub mod pipeline;

pub(crate) use crate::vk_swapchain::Swapchain;
pub(crate) use crate::vk_resmgr::ResourceManager;
pub(crate) use crate::vk_syncmgr::SynchronizationManager;
pub(crate) use crate::vk_deskmgr::DescriptorManager;
pub(crate) use crate::vk_renderpassmgr::RenderPassManager;
pub(crate) use crate::vk_subpassmgr::SubpassManager;
pub(crate) use crate::pipeline::PipelineManager;

use ash::{
    vk,
    extensions::khr
};

#[allow(dead_code)]
pub struct VulkanQueue<'a> {
    device: &'a ash::Device,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    buffer: vk::Buffer,
    frame_buffer: u32,
    swapchain: Swapchain,
    resource_manager: ResourceManager,
    descriptor_manager: DescriptorManager,
    sync_manager: SynchronizationManager,
    pipeline_manager: PipelineManager,
    subpass_manager: SubpassManager,
    render_pass_manager: RenderPassManager,
}

#[allow(unused_must_use)]
#[allow(dead_code)]
impl<'a> VulkanQueue<'a> {
    pub fn new(
        instance: &ash::Instance,
        device: &'a ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &khr::Surface,
        queue_index: u32,
        buffer: vk::Buffer,
        frame_buffer: u32,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device.create_command_pool(&command_pool_create_info, None).unwrap()
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_index, 0) };

        let swapchain = Swapchain::new(
            instance,
            device,
            physical_device,
            surface,
            window_width,
            window_height,
            surface_loader,
        );

        let memory_properties = unsafe {
            instance.get_physical_device_memory_properties(physical_device)
        };
        let resource_manager = ResourceManager::new(device.clone(), memory_properties);

        let sync_manager = SynchronizationManager::new(device.clone());

        let descriptor_manager = DescriptorManager::new(device.clone());

        let pipeline_manager = PipelineManager::new(device.clone());

        let subpass_manager = SubpassManager::new();

        let render_pass_manager = RenderPassManager::new(device.clone());

        Self {
            device,
            command_pool,
            graphics_queue,
            buffer,
            frame_buffer,
            swapchain,
            resource_manager,
            sync_manager,
            descriptor_manager,
            pipeline_manager,
            subpass_manager,
            render_pass_manager,
        }
    }

    pub async fn submit_commands(
        &self,
        num_threads: usize,
        mut command_generator: impl FnMut(usize) -> Vec<vk::CommandBuffer> + Send + 'static,
    ) -> Result<(), vk::Result> {
        let mut fence_vec = Vec::with_capacity(num_threads);
        let mut semaphore_vec = Vec::with_capacity(num_threads);
    
        for _ in 0..num_threads {
            fence_vec.push(unsafe { self.device.create_fence(&vk::FenceCreateInfo::default(), None).unwrap() });
            semaphore_vec.push(unsafe { self.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).unwrap() });
        }
    
        let mut futures = Vec::with_capacity(num_threads);
    
        for i in 0..num_threads {
            let command_buffers = command_generator(i);
            let command_buffers_len = command_buffers.len() as u32;
    
            let (synced_tx, _synced_rx) = tokio::sync::oneshot::channel();
    
            unsafe { self.device.reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::RELEASE_RESOURCES); }
            let (tx, mut rx) = tokio::sync::mpsc::channel(command_buffers_len as usize);
            let device = self.device.clone();
            let graphics_queue = self.graphics_queue;
            let fence = fence_vec[i];
            let semaphore = semaphore_vec[i];
    
            let future = tokio::spawn(async move {
                for command_buffer in &command_buffers {
                    tx.send(command_buffer.clone()).await.unwrap();
                }
                drop(tx);
    
                let mut command_buffer_handles = Vec::new();
                while let Some(command_buffer) = rx.recv().await {
                    command_buffer_handles.push(command_buffer);
                }
    
                let submit_infos: Vec<_> = command_buffer_handles.iter().map(|c| vk::SubmitInfo::builder().command_buffers(&[*c]).wait_semaphores(&[semaphore]).signal_semaphores(&[semaphore]).build()).collect();
    
                unsafe {
                    device.queue_submit(graphics_queue, submit_infos.as_slice(), fence);
                }                
    
                synced_tx.send(()).unwrap();
    
                Ok::<(), vk::Result>(())
            });
    
            futures.push(future);
        }
    
        for future in futures {
            future.await.unwrap();
        }
    
        let sync_submit_info = vk::SubmitInfo::builder().wait_semaphores(&semaphore_vec).wait_dst_stage_mask(&[vk::PipelineStageFlags::ALL_COMMANDS]).build();
    
        unsafe {
            self.device.queue_submit(self.graphics_queue, &[sync_submit_info], vk::Fence::null());
        }
    
        for fence in &fence_vec {
            let timeout = std::time::Duration::from_millis(1000);
    
            match unsafe {
                self.device
                    .wait_for_fences(&[*fence], true, timeout.as_nanos() as u64)
            } {
                Ok(_) => {}
                Err(error) => {
                    return Err(error);
                }
            }
        }
    
        Ok(())
    }    
}
