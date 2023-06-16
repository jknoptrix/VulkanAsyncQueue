use ash::vk;

pub struct ComputePipelineManager {
    device: ash::Device,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    pipeline_layouts: Vec<vk::PipelineLayout>,
    pipelines: Vec<vk::Pipeline>,
}

impl ComputePipelineManager {
    pub fn new(device: ash::Device) -> Self {
        Self {
            device,
            descriptor_set_layouts: Vec::new(),
            pipeline_layouts: Vec::new(),
            pipelines: Vec::new(),
        }
    }

    pub fn create_compute_pipeline(
        &mut self,
        shader_module: vk::ShaderModule,
        descriptor_set_layout_bindings: &[vk::DescriptorSetLayoutBinding],
    ) -> vk::Pipeline {
        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(descriptor_set_layout_bindings);
    
        let descriptor_set_layout = unsafe {
            self.device
                .create_descriptor_set_layout(&descriptor_set_layout_create_info, None)
                .unwrap()
        };
        let binding = [descriptor_set_layout];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&binding);
    
        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .unwrap()
        };
    
        let shader_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap());
    
        let compute_pipeline_create_info = vk::ComputePipelineCreateInfo::builder()
            .stage(shader_stage_create_info.build())
            .layout(pipeline_layout);
    
        let pipeline = unsafe {
            self.device
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    &[compute_pipeline_create_info.build()],
                    None,
                )
                .unwrap()[0]
        };
    
        self.descriptor_set_layouts.push(descriptor_set_layout);
        self.pipeline_layouts.push(pipeline_layout);
        self.pipelines.push(pipeline);
    
        pipeline
    }    
}
