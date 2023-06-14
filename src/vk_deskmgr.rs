use ash::vk;

pub struct DescriptorManager {
    device: ash::Device,
}

impl DescriptorManager {
    pub fn new(device: ash::Device) -> Self {
        Self { device }
    }

    pub fn create_descriptor_pool(
        &self,
        max_sets: u32,
        pool_sizes: &[vk::DescriptorPoolSize],
    ) -> vk::DescriptorPool {
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(max_sets)
            .build();

        unsafe { self.device.create_descriptor_pool(&pool_info, None).unwrap() }
    }

    pub fn create_descriptor_set_layout(
        &self,
        bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> vk::DescriptorSetLayout {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();

        unsafe { self.device.create_descriptor_set_layout(&layout_info, None).unwrap() }
    }

    pub fn create_descriptor_sets(
        &self,
        pool: vk::DescriptorPool,
        layouts: &[vk::DescriptorSetLayout],
    ) -> Vec<vk::DescriptorSet> {
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(layouts)
            .build();

        unsafe { self.device.allocate_descriptor_sets(&allocate_info).unwrap() }
    }

    pub fn update_descriptor_sets(&self, writes: &[vk::WriteDescriptorSet]) {
        unsafe { self.device.update_descriptor_sets(writes, &[]) };
    }
}
