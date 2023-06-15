use ash::vk;

pub struct MemoryManager {
    device: ash::Device,
    memory_prop: vk::PhysicalDeviceMemoryProperties,
}

impl MemoryManager {
    pub fn new(
        device: ash::Device,
        memory_prop: vk::PhysicalDeviceMemoryProperties,
    ) -> Self {
        Self {
            device,
            memory_prop,
        }
    }

    pub fn allocate_memory(
        &self,
        allocation_size: vk::DeviceSize,
        memory_type_index: u32,
    ) -> vk::DeviceMemory {
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(allocation_size)
            .memory_type_index(memory_type_index)
            .build();

        unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .expect("Failed to allocate memory")
        }
    }

    pub fn free_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.device.free_memory(memory, None);
        }
    }

    pub fn map_memory<T>(
        &self,
        memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> *mut T {
        unsafe {
            self.device
                .map_memory(memory, offset, size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut T
        }
    }

    pub fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.device.unmap_memory(memory);
        }
    }

    pub fn copy_data_to_memory<T>(
        &self,
        data: &[T],
        memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
    ) {
        let size = (data.len() * std::mem::size_of::<T>()) as vk::DeviceSize;
        let mapped_memory = self.map_memory::<T>(memory, offset, size);

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), mapped_memory, data.len());
            self.device.unmap_memory(memory);
        }
    }

    pub fn find_memory_type_index(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        for i in 0..self.memory_prop.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && self.memory_prop.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Some(i);
            }
        }

        None
    }
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            memory_prop: self.memory_prop,
        }
    }
}
