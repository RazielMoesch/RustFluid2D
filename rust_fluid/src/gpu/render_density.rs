use crate::gpu::utils::{FRAGMENT, storage_buffer, uniform_buffer};

pub struct RenderDensity {
    pub bgl0: wgpu::BindGroupLayout,
    pub bgl1: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline
}

impl RenderDensity {

    pub fn new( device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> Self {

        let bgl0 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("RD BGL0"),
                entries: &[
                    uniform_buffer(0, FRAGMENT)
                ]
            }
        );

        let bgl1 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("RD BGL1"),
                entries: &[
                    storage_buffer(0, FRAGMENT, true),
                    storage_buffer(1, FRAGMENT, true)
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu:: PipelineLayoutDescriptor {
                label: Some("RD Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&bgl0),
                    Some(&bgl1),
                ],
                immediate_size: 0
            }
        );

        let module = device.create_shader_module(wgpu::include_wgsl!("shaders/render_density.wgsl"));

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("RD Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[]
                },
                fragment: Some(
                    wgpu::FragmentState {
                        module: &module,
                        entry_point: Some("fs"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[
                            Some(
                                wgpu::ColorTargetState {
                                    format: config.format,
                                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                    write_mask: wgpu::ColorWrites::ALL
                                }
                            )
                        ]
                    }
                ),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
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
        view: &wgpu::TextureView,
        bg0: &wgpu::BindGroup,
        bg1: &wgpu::BindGroup,

    ) {

        let mut pass  = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                 label: Some("RD Render Pass"),
                 color_attachments: &[
                    Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store
                            },
                            depth_slice: None
                        }
                    )
                 ],
                 depth_stencil_attachment: None,
                 timestamp_writes: None,
                 occlusion_query_set: None,
                 multiview_mask: None
            }
        );

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, bg0, &[]);
        pass.set_bind_group(1, bg1, &[]);

        pass.draw(0..3, 0..1);

    }

}