use ash::vk;

pub struct ResourceManager {
    device: ash::Device,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl ResourceManager {
    pub fn new(device: ash::Device, memory_properties: vk::PhysicalDeviceMemoryProperties) -> Self {
        Self {
            device,
            memory_properties,
        }
    }

    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { self.device.create_buffer(&buffer_info, None).unwrap() };

        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let memory_type = self.find_memory_type(mem_requirements.memory_type_bits, properties);

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type)
            .build();

        let buffer_memory = unsafe { self.device.allocate_memory(&allocate_info, None).unwrap() };
        unsafe { self.device.bind_buffer_memory(buffer, buffer_memory, 0).unwrap() };

        (buffer, buffer_memory)
    }

    pub fn create_image(
        &self,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> (vk::Image, vk::DeviceMemory) {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build();

        let image = unsafe { self.device.create_image(&image_info, None).unwrap() };

        let mem_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let memory_type = self.find_memory_type(mem_requirements.memory_type_bits, properties);

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type)
            .build();

        let image_memory = unsafe { self.device.allocate_memory(&allocate_info, None).unwrap() };
        unsafe { self.device.bind_image_memory(image, image_memory, 0).unwrap() };

        (image, image_memory)
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> u32 {
        for i in 0..self.memory_properties.memory_type_count {
            if type_filter & (1 << i) != 0
                && self.memory_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return i;
            }
        }

        panic!("Failed to find suitable memory type.")
    }
}
