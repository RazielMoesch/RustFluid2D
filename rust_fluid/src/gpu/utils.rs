

pub const COMPUTE: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
pub const VERTEX: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX;
pub const FRAGMENT: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;

pub fn storage_buffer(binding: u32, visibility: wgpu::ShaderStages, read_only: bool) -> wgpu::BindGroupLayoutEntry {

    wgpu::BindGroupLayoutEntry  {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: read_only },
            has_dynamic_offset: false,
            min_binding_size: None
        },
        count: None
    }

}


pub fn uniform_buffer(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: binding,
        visibility: visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None
        },
        count: None
    }
}


pub fn create_buffer_init(device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(
        &wgpu::BufferDescriptor {
            label: None,
            size: size,
            usage: usage,
            mapped_at_creation: false
        }
    )
}



pub fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, entries: &[wgpu::BindGroupEntry]) -> wgpu::BindGroup {
    device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: None,
            layout: layout,
            entries: entries
        }
    )
}

pub fn buffer_binding_entry(buffer: &wgpu::Buffer, binding: u32) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource: buffer.as_entire_binding() }
}




