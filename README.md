# Vulkan Queue

This shit provides a simple and easy-to-use interface for working with Vulkan queues and swapchains in Rust.

## Features

- Asynchronous queue submission using Tokio
- Easy swapchain creation and management
- Support for multiple threads

## Usage
You can create a new `VulkanQueue` instance and use it to submit command buffers to the graphics queue:

```rust
let queue = VulkanQueue::new(
    &instance,
    &device,
    physical_device,
    surface,
    &surface_loader,
    queue_index,
    buffer,
    frame_buffer,
    window_width,
    window_height
);

let command_generator = |i: usize| {
    // Generate command buffers here...
};

queue.submit_commands(1, command_generator).await.unwrap();
```

In this example, we create a new `VulkanQueue` instance and pass in the necessary parameters, such as the Vulkan instance and device handles, the physical device, the surface, and the surface loader. We also specify the queue family index, buffer, frame buffer, window width, and window height.

Then we create a closure that generates command buffers for submission to the graphics queue. This closure takes an index as an argument and returns a vector of command buffers. Then we call the `submit_commands` method on our `VulkanQueue` instance and pass in our command generator closure. This will submit our command buffers to the graphics queue for execution.

# Swapchain
This bullshit also provides support for creating and managing swapchains. When you create a new `VulkanQueue` instance, a default swapchain is automatically created for you using the specified parameters.

You can access the swapchain images for rendering by using the `swapchain.images` field of your `VulkanQueue` instance:
```rust
let framebuffers: Vec<vk::Framebuffer> = queue.swapchain
    .images
    .iter()
    .map(|&image| {
        // create framebuffers here...
    })
    .collect();
```
Here we iterating over the `swapchain` images and use them to create framebuffers for rendering.

Once you’ve rendered your scene to a `swapchain` image, you can present it to the surface using the `queue_present_khr` method of the `Swapchain` object:
```rust
let present_info = vk::PresentInfoKHR::builder()
    .wait_semaphores(&[semaphore])
    .swapchains(&[queue.swapchain.handle])
    .image_indices(&[image_index])
    .build();

unsafe {
    queue.swapchain.loader.queue_present(queue.graphics_queue, &present_info).unwrap();
}
```
`PresentInfoKHR` struct specifies which swapchain and image to present, as well as which semaphore to wait on before presenting. Then we call the `queue_present_khr` method on our `Swapchain` object to present the specified image to the surface.

# Examples

### Commands (rendering triangles):
```rust
let pipeline = create_pipeline(device, swapchain.image_format);
let command_generator = |i: usize| {
    let command_buffer = create_command_buffer(device, command_pool);

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .build();
    unsafe {
        device
            .begin_command_buffer(command_buffer, &begin_info)
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    let clear_values = [vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    }];

    let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        .render_pass(render_pass)
        .framebuffer(framebuffers[i])
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        })
        .clear_values(&clear_values)
        .build();

    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        );
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
        device.cmd_draw(command_buffer, 3, 1, 0, 0);
        device.cmd_end_render_pass(command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to record Command Buffer at Ending!");
    }

    vec![command_buffer]
};
queue.submit_commands(1, command_generator).await.unwrap();
```
### Framebuffers using swapchain:
You can extend it by yourself, but you can already call it and 'setting this shit up' the queue settings:
```rust
let framebuffers: Vec<vk::Framebuffer> = swapchain
    .images
    .iter()
    .map(|&image| {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(swapchain.image_format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image(image)
            .build();

        let image_view = unsafe {
            device
                .create_image_view(&image_view_create_info, None)
                .unwrap()
        };

        let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&[image_view])
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1)
            .build();

        unsafe { device.create_framebuffer(&framebuffer_create_info, None).unwrap() }
    })
    .collect();

// ...
let present_info = vk::PresentInfoKHR::builder()
    .wait_semaphores(&[semaphore])
    .swapchains(&[swapchain.handle])
    .image_indices(&[image_index])
    .build();

unsafe {
    swapchain.loader.queue_present(queue.graphics_queue, &present_info).unwrap();
}
```

### Resource managment:
```rust
let vertex_data = [/* ... */];
let buffer_size = (vertex_data.len() * std::mem::size_of::<Vertex>()) as u64;

let (vertex_buffer, vertex_buffer_memory) = queue.resource_manager.create_buffer(
    buffer_size,
    vk::BufferUsageFlags::VERTEX_BUFFER,
    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
);

let data_ptr = unsafe {
    device
        .map_memory(vertex_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
        .unwrap()
};
unsafe {
    std::ptr::copy_nonoverlapping(vertex_data.as_ptr() as *const u8, data_ptr, buffer_size as usize);
    device.unmap_memory(vertex_buffer_memory);
}
```