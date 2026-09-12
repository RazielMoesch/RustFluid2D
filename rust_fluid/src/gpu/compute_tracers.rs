use crate::gpu::utils::{COMPUTE, storage_buffer, uniform_buffer};





pub struct ComputeTracers {
    pub bgl0: wgpu::BindGroupLayout,
    pub bgl1: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline
}


impl ComputeTracers {

    pub fn new(device: &wgpu::Device) -> Self {

        let bgl0 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("CT BGL0"),
                entries: &[
                    uniform_buffer(0, COMPUTE)
                ]
            }
        );

        let bgl1 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("CT BGL1"),
                entries: &[
                    storage_buffer(0, COMPUTE, true),
                    storage_buffer(1, COMPUTE, false),
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("CT Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&bgl0),
                    Some(&bgl1)
                ],
                immediate_size: 0
            }
        );

        let module = device.create_shader_module(wgpu::include_wgsl!("shaders/compute_tracers.wgsl"));

        let pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("CT Pipeline"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some("compute_tracers"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None
            }
        );


        Self {
            bgl0,
            bgl1,
            pipeline
        }
    }


    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bg0: &wgpu::BindGroup,
        bg1: &wgpu::BindGroup,
        num_workgroups: u32
    ) -> () {

        let mut pass = encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor::default()
        );

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, bg0, &[]);
        pass.set_bind_group(1, bg1, &[]);

        pass.dispatch_workgroups(num_workgroups, 1, 1);

    }

}

