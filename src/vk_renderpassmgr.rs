use ash::vk;

pub struct RenderPassManager {
    device: ash::Device,
}

impl RenderPassManager {
    pub fn new(device: ash::Device) -> Self {
        Self { device }
    }

    pub fn create_render_pass(
        &self,
        attachments: &[vk::AttachmentDescription],
        subpasses: &[vk::SubpassDescription],
        dependencies: &[vk::SubpassDependency],
    ) -> vk::RenderPass {
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies)
            .build();

        unsafe { self.device.create_render_pass(&render_pass_info, None).unwrap() }
    }

    pub fn create_framebuffer(
        &self,
        render_pass: vk::RenderPass,
        attachments: &[vk::ImageView],
        width: u32,
        height: u32,
        layers: u32,
    ) -> vk::Framebuffer {
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(width)
            .height(height)
            .layers(layers)
            .build();

        unsafe { self.device.create_framebuffer(&framebuffer_info, None).unwrap() }
    }
}
