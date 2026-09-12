use crate::gpu::utils::{COMPUTE, storage_buffer, uniform_buffer};




pub struct ComputeInitialConditions {

    pub bgl0: wgpu::BindGroupLayout,
    pub bgl1: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline

}

impl ComputeInitialConditions {

    pub fn new( device: &wgpu::Device ) -> Self {
        

        let bgl0 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("CIC BGL0"),
                entries: &[
                    uniform_buffer(0, COMPUTE)
                ]
            }
        );

        let bgl1 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("CIC BGL1"),
                entries: &[
                    storage_buffer(0, COMPUTE, false),
                    storage_buffer(1, COMPUTE, false),
                    storage_buffer(2, COMPUTE, false),
                    storage_buffer(3, COMPUTE, false),
                    storage_buffer(4, COMPUTE, false),
                    storage_buffer(5, COMPUTE, true),
                    storage_buffer(6, COMPUTE, false)
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("CIC Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&bgl0),
                    Some(&bgl1)
                ],
                immediate_size: 0
            }
        );

        let module = device.create_shader_module(wgpu::include_wgsl!("shaders/compute_initial_conditions.wgsl"));

        let pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("CIC Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some("compute_initial_conditions"),
                cache: None,
                compilation_options: wgpu::PipelineCompilationOptions::default()
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
        wg_x: u32, wg_y: u32
    ) {
        let mut pass = encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor {
                label: Some("Compute Initial Conditions Pass"),
                ..Default::default()
            }
        );

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg0, &[]);
        pass.set_bind_group(1, bg1, &[]);
        pass.dispatch_workgroups(wg_x, wg_y, 1);

    }

}

