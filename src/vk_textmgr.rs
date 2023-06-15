use ash::vk;
use std::path::Path;
use image::GenericImageView;

use crate::vk_memorymgr::MemoryManager;

pub struct TextureManager {
    device: ash::Device,
    memory_manager: MemoryManager,
}

impl TextureManager {
    pub fn new(device: ash::Device, memory_manager: MemoryManager) -> Self {
        Self {
            device,
            memory_manager,
        }
    }

    pub fn load_texture(
        &self,
        file_path: &Path,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        aspect_mask: vk::ImageAspectFlags,
    ) -> (vk::Image, vk::ImageView) {
        let img = image::open(file_path).expect("Failed to open texture file");
        let (width, height) = img.dimensions();
        let data = img.to_rgba8().into_raw();

        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            self.device
                .create_image(&image_create_info, None)
                .expect("Failed to create image")
        };

        let mem_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let memory_type_index = self
            .memory_manager
            .find_memory_type_index(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Failed to find suitable memory type for image");

        let memory = self.memory_manager.allocate_memory(mem_requirements.size, memory_type_index);
        unsafe {
            self.device
                .bind_image_memory(image, memory, 0)
                .expect("Failed to bind image memory");
        }

        self.memory_manager.copy_data_to_memory(
            &data,
            memory,
            0,
        );

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            );

        let view = unsafe {
            self.device
                .create_image_view(&view_create_info, None)
                .expect("Failed to create image view")
        };

        (image, view)
    }
}
