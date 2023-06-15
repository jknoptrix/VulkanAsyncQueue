use ash::vk;
use std::io::Cursor;

use std::path::Path;
pub struct ShaderManager {
    device: ash::Device,
}

impl ShaderManager {
    pub fn new(device: ash::Device) -> Self {
        Self { device }
    }

    pub fn load_shader_module(&self, file_path: &Path) -> vk::ShaderModule {
        let code = std::fs::read(file_path).expect("Failed to read shader file");
        let code = ash::util::read_spv(&mut Cursor::new(code)).expect("Failed to read shader code");

        let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

        unsafe {
            self.device
                .create_shader_module(&shader_info, None)
                .expect("Failed to create shader module")
        }
    }
}
