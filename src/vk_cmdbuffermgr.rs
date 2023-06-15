use ash::{
    vk,
};

pub struct CommandBufferManager {
    device: ash::Device,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandBufferManager {
    pub fn new(device: ash::Device, command_pool: vk::CommandPool) -> Self {
        Self {
            device,
            command_pool,
            command_buffers: Vec::new(),
        }
    }
    
    pub fn begin_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin command buffer");
        }
    }

    pub fn allocate_command_buffers(&mut self, count: u32) -> &[vk::CommandBuffer] {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();

        self.command_buffers = unsafe {
            self.device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers")
        };

        &self.command_buffers
    }

    pub fn free_command_buffers(&mut self) {
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &self.command_buffers);
        }
        self.command_buffers.clear();
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer");
        }
    }
}
