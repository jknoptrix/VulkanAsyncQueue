use ash::vk;

pub struct SubpassManager {}

impl SubpassManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_subpass_description(
        &self,
        pipeline_bind_point: vk::PipelineBindPoint,
        input_attachments: &[vk::AttachmentReference],
        color_attachments: &[vk::AttachmentReference],
        resolve_attachments: Option<&[vk::AttachmentReference]>,
        depth_stencil_attachment: Option<&vk::AttachmentReference>,
        preserve_attachments: &[u32],
    ) -> vk::SubpassDescription {
        vk::SubpassDescription::builder()
            .pipeline_bind_point(pipeline_bind_point)
            .input_attachments(input_attachments)
            .color_attachments(color_attachments)
            .resolve_attachments(resolve_attachments.unwrap_or_default())
            .depth_stencil_attachment(depth_stencil_attachment.unwrap_or(&vk::AttachmentReference::default()))
            .preserve_attachments(preserve_attachments)
            .build()
    }
}