use wgpu::util::DeviceExt;

pub fn create_buffer_init( device: &wgpu::Device, contents: &[u8], usage: wgpu::BufferUsages ) -> wgpu::Buffer {
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage
        }
    )
}

pub fn create_buffer( device: &wgpu::Device, size: u64, usage: wgpu::BufferUsages ) -> wgpu::Buffer {
    device.create_buffer(
        &wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false
        }
    )
}

pub fn create_bind_group( device: &wgpu::Device, layout: &wgpu::BindGroupLayout, entries: &[wgpu::BindGroupEntry] ) -> wgpu::BindGroup {
    device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries
        }
    )
}

pub fn buffer_binding_entry( buffer: &wgpu::Buffer , binding: u32 ) -> wgpu::BindGroupEntry<'_> {

    wgpu::BindGroupEntry { binding, resource: buffer.as_entire_binding() }

}
