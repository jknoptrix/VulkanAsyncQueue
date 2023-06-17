use std::ffi::c_void;
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::sync::Arc;

use ash::extensions::khr::*;
use ash::vk;

use crate::memtype::find_memorytype_index;

pub struct RTPipelineManager {
    device: Arc<ash::Device>,
    ray_tracing_pipeline_properties: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_create_infos: Vec<vk::DeviceQueueCreateInfo>,
    physical_device: vk::PhysicalDevice,
}

impl RTPipelineManager {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        queue_create_infos: Vec<vk::DeviceQueueCreateInfo>,
    ) -> Self {
        let ray_tracing_pipeline_properties = Self::get_ray_tracing_properties(entry, instance, physical_device);
        let device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        Self {
            device,
            ray_tracing_pipeline_properties,
            device_memory_properties,
            queue_create_infos,
            physical_device,
        }
    }

    pub fn get_ray_tracing_properties(
        entry: &ash::Entry,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceRayTracingPipelinePropertiesKHR {
        let mut ray_tracing_pipeline_props = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::builder().build();
        let mut properties2 = vk::PhysicalDeviceProperties2KHR::builder()
            .push_next(&mut ray_tracing_pipeline_props)
            .build();
        unsafe {
            instance.get_physical_device_properties2(physical_device, &mut properties2);   
            *properties2.p_next.cast::<vk::PhysicalDeviceRayTracingPipelinePropertiesKHR>()
        }
    }

    pub fn create_shader_module(&self, code: &[u8]) -> vk::ShaderModule {
        let code = ash::util::read_spv(&mut std::io::Cursor::new(code)).unwrap();
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(&code);
        unsafe { self.device.create_shader_module(&shader_module_create_info, None).unwrap() }
    }

    pub fn create_ray_tracing_pipeline(
        &self,
        instance: &ash::Instance,
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
        groups: &[vk::RayTracingShaderGroupCreateInfoKHR],
        max_recursion_depth: u32,
        layout: vk::PipelineLayout,
    ) -> vk::Pipeline {
        let ray_tracing_pipeline = RayTracingPipeline::new(instance, &self.device);
    
        let pipeline_create_info = vk::RayTracingPipelineCreateInfoKHR::builder()
            .stages(shader_stages)
            .groups(groups)
            .max_pipeline_ray_recursion_depth(max_recursion_depth)
            .layout(layout);
    
        let pipeline = unsafe {
            ray_tracing_pipeline
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    vk::PipelineCache::null(),
                    &[pipeline_create_info.build()],
                    None,
                )
                .unwrap()[0]
        };
        pipeline
    }

    pub fn create_shader_binding_table(
        &self,
        instance: &ash::Instance,
        pipeline: vk::Pipeline,
        groups: &[vk::RayTracingShaderGroupCreateInfoKHR],
        group_handle_size: u32,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let group_count = groups.len() as u32;
        let sbt_size = group_count * group_handle_size;

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(sbt_size as u64)
            .usage(vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS);

        let buffer = unsafe { self.device.create_buffer(&buffer_create_info, None).unwrap() };

        let memory_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        
         // Find a memory type that is host visible and coherent
         let memory_type_index = find_memorytype_index(
            &memory_requirements,
            &self.device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .expect("Unable to find suitable memory type for the shader binding table.");        

         // Allocate memory and bind it to the buffer
         let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

         let memory = unsafe {
             self.device
                 .allocate_memory(&allocate_info, None)
                 .expect("Failed to allocate memory for the shader binding table.")
         };
         unsafe {
             self.device
                 .bind_buffer_memory(buffer, memory, 0)
                 .expect("Failed to bind memory to the shader binding table buffer.");
         }

        let data = unsafe {
            self.device
                .map_memory(
                    memory,
                    0,
                    sbt_size as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap()
        };

        let ray_tracing_pipeline = RayTracingPipeline::new(instance, &self.device);

        let group_handles = unsafe {
            ray_tracing_pipeline
                .get_ray_tracing_shader_group_handles(
                    pipeline,
                    0,
                    group_count,
                    sbt_size as usize,
                )
                .unwrap()
        };

        unsafe {
            data.copy_from_nonoverlapping(group_handles.as_ptr() as *const c_void, sbt_size as usize);
            self.device.unmap_memory(memory);
        }

        (buffer, memory)
    }
    
    pub fn create_bottom_level_acceleration_structure(
        &self,
        instance: &ash::Instance,
        geometries: &[vk::GeometryNV],
        flags: vk::BuildAccelerationStructureFlagsNV,
    ) -> (vk::AccelerationStructureNV, vk::DeviceMemory) {
        let device_extensions = [ash::extensions::nv::RayTracing::name().as_ptr()];
    
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&self.queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .build();
    
        let device = unsafe {
            instance
                .create_device(self.physical_device, &device_create_info, None)
                .expect("Failed to create logical device.")
        };
    
        let ray_tracing = ash::extensions::nv::RayTracing::new(instance, &device);
        let ray_tracing_fn = vk::NvRayTracingFn::load(|name| unsafe {
            mem::transmute(instance.get_device_proc_addr(device.handle(), name.as_ptr()))
        });
    
        let create_info = vk::AccelerationStructureCreateInfoNV::builder()
            .info(
                vk::AccelerationStructureInfoNV::builder()
                    .ty(vk::AccelerationStructureTypeNV::BOTTOM_LEVEL)
                    .flags(flags)
                    .geometries(geometries)
                    .build(),
            )
            .build();
    
        let mut acceleration_structure = MaybeUninit::<vk::AccelerationStructureNV>::uninit();
        unsafe {
            (ray_tracing_fn.create_acceleration_structure_nv)(
                device.handle(),
                &create_info,
                ptr::null(),
                acceleration_structure.as_mut_ptr(),
            )
            .result()
            .unwrap();
        }
        let acceleration_structure = unsafe { acceleration_structure.assume_init() };
    
        let mut memory_requirements = MaybeUninit::<vk::MemoryRequirements2>::uninit();
        unsafe {
            (ray_tracing_fn.get_acceleration_structure_memory_requirements_nv)(
                device.handle(),
                &vk::AccelerationStructureMemoryRequirementsInfoNV::builder()
                    .acceleration_structure(acceleration_structure)
                    .ty(vk::AccelerationStructureMemoryRequirementsTypeNV::OBJECT)
                    .build(),
                memory_requirements.as_mut_ptr(),
            );
        }
        let memory_requirements = unsafe { memory_requirements.assume_init() }.memory_requirements;
    
        let memory_type_index = find_memorytype_index(
            &memory_requirements,
            &self.device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find suitable memory type for the acceleration structure.");
    
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);
    
        let memory = unsafe {
            device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate memory for the acceleration structure.")
        };
    
        let bind_info = vk::BindAccelerationStructureMemoryInfoNV::builder()
            .acceleration_structure(acceleration_structure)
            .memory(memory)
            .build();
    
        unsafe {
            (ray_tracing_fn.bind_acceleration_structure_memory_nv)(
                device.handle(),
                1,
                &bind_info,
            )
            .result()
            .unwrap();
        }
    
        (acceleration_structure, memory)
    }            
}
