use crate::gpu::utils::{VERTEX, storage_buffer, uniform_buffer};

pub struct RenderFlowStreams {
    pub bgl0: wgpu::BindGroupLayout,
    pub bgl1: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline
}

impl RenderFlowStreams {

    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {

        let bgl0 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("RFS BGL0"),
                entries: &[
                    uniform_buffer(0, VERTEX),
                ]
            }
        );

        let bgl1 = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("RFS BGL1"),
                entries: &[
                    storage_buffer(0, VERTEX, true)
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("RFS Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&bgl0),
                    Some(&bgl1)
                ],
                immediate_size: 0
            }
        );

        let module = device.create_shader_module(wgpu::include_wgsl!("shaders/render_flow_streams.wgsl"));

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("RFS Pipeline"),
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
                    cull_mode: None,
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
        num_vertices: u32
    )  {

        let mut pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("RFS Render Pass"),
                color_attachments: &[
                    Some(
                        wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store
                            }
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
        pass.draw(0..6, 0..num_vertices);

    }

}