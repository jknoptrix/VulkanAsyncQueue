# Vulkan Async Queue
VulkanQueue is a Rust struct that represents a queue for submitting commands to a Vulkan device. It contains references to the Vulkan device, a command pool, a graphics queue, a buffer, and a frame buffer.

# Usage
To create a new instance of `VulkanQueue`, call the `new` function and provide it with references to the Vulkan device, the queue family index, the buffer, and the frame buffer value:
```rust
let queue = VulkanQueue::new(device, queue_index, buffer, frame_buffer);
```
To submit commands to the queue, call the submit_commands function and provide it with the number of threads to use for command generation and a closure that generates command buffers:
```rust
queue.submit_commands(num_threads, |i| {
    // generate command buffers for thread i
    vec![/* command buffers */]
});
```
The `submit_commands` function will create a vector of fences and semaphores with a capacity equal to the number of threads. It will then spawn a future for each thread that generates command buffers and submits them to the queue. The function will wait for all futures to complete before submitting a synchronization command to the queue and waiting for all fences to be signaled before returning.

Also this project has included simulation of queue configuration:
```rust
pub struct AsyncQueue;

pub struct Compute {
    pub state: bool,
    pub buffer: u32,
}

impl Compute {
    pub fn new(state: bool, buffer: u32) -> Self {
        Self { state, buffer }
    }
}

impl Default for Compute {
    fn default() -> Self {
        Self {
            state: true,
            buffer: 1024,
        }
    }
}
```
You can extend it by yourself, but you can already call it and 'setting this shit up' the queue settings:
```rust
fn main() {
    let queue = Compute::new(
        true, // <-- states
        1024 // <-- buffer
    );
}
```
