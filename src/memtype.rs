
use ash::vk;

pub fn find_memorytype_index(
    requirements: &vk::MemoryRequirements,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    usage: vk::MemoryPropertyFlags,
) -> Option<u32> {
    for i in 0..memory_properties.memory_type_count {
        if requirements.memory_type_bits & (1 << i) != 0
            && memory_properties.memory_types[i as usize].property_flags.contains(usage)
        {
            return Some(i);
        }
    }
    None
}
