use ash::vk;

pub struct PipelineManager {
    device: ash::Device,
}

impl PipelineManager {
    pub fn new(device: ash::Device) -> Self {
        Self { device }
    }

    pub fn create_shader_module(&self, code: &[u8]) -> vk::ShaderModule {
        let code = ash::util::read_spv(&mut std::io::Cursor::new(code)).unwrap();
        let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code).build();
        unsafe { self.device.create_shader_module(&shader_info, None).unwrap() }
    }

    pub fn create_graphics_pipeline(
        &self,
        vertex_shader: vk::ShaderModule,
        fragment_shader: vk::ShaderModule,
        vertex_input_info: &vk::PipelineVertexInputStateCreateInfo,
        input_assembly_info: &vk::PipelineInputAssemblyStateCreateInfo,
        viewport_info: &vk::PipelineViewportStateCreateInfo,
        rasterization_info: &vk::PipelineRasterizationStateCreateInfo,
        multisample_info: &vk::PipelineMultisampleStateCreateInfo,
        color_blend_info: &vk::PipelineColorBlendStateCreateInfo,
        depth_stencil_info: Option<&vk::PipelineDepthStencilStateCreateInfo>,
        layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
    ) -> vk::Pipeline {
        let shader_entry_name = std::ffi::CString::new("main").unwrap();
    
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader)
                .name(&shader_entry_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader)
                .name(&shader_entry_name)
                .build(),
        ];
    
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(vertex_input_info)
            .input_assembly_state(input_assembly_info)
            .viewport_state(viewport_info)
            .rasterization_state(rasterization_info)
            .multisample_state(multisample_info)
            .color_blend_state(color_blend_info)
            .depth_stencil_state(depth_stencil_info.unwrap_or(&vk::PipelineDepthStencilStateCreateInfo::default()))
            .layout(layout)
            .render_pass(render_pass)
            .build();
    
        unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .unwrap()[0]
        }
    }
        
}
