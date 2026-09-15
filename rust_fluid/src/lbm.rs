
use crate::{
    gpu::{
        compute_initial_conditions::ComputeInitialConditions,
        compute_macros::ComputeMacros,
        compute_step::ComputeStep,
        compute_tracers::ComputeTracers,
        render_curl::RenderCurl,
        render_density::RenderDensity,
        render_flow_streams::RenderFlowStreams,
        render_speed::RenderSpeed,
        render_tracers::RenderTracers,
        render_x_velocity::RenderXVelocity,
        render_y_velocity::RenderYVelocity,
        resources::Uniforms,
        utils::{buffer_binding_entry, create_bind_group},
    },
    utils::{create_buffer, create_buffer_init},
};

pub struct LBMBuffers {
    pub uniforms: wgpu::Buffer,
    pub fa: wgpu::Buffer,
    pub fb: wgpu::Buffer,
    pub types: wgpu::Buffer,
    pub density: wgpu::Buffer,
    pub veloctiy: wgpu::Buffer,
    pub object: wgpu::Buffer,
    pub tracers: wgpu::Buffer,
}

pub struct ComputeStages {
    pub init: ComputeInitialConditions,
    pub macros: ComputeMacros,
    pub step: ComputeStep,
    pub tracers: ComputeTracers,
}

pub struct RenderStages {
    pub density: RenderDensity,
    pub speed: RenderSpeed,
    pub x_vel: RenderXVelocity,
    pub y_vel: RenderYVelocity,
    pub curl: RenderCurl,
    pub tracers: RenderTracers,
    pub flow_streams: RenderFlowStreams,
}

pub struct BindGroups {
    pub bg0: wgpu::BindGroup,
    pub bg1: wgpu::BindGroup,
}

pub struct LBM {
    pub buffers: LBMBuffers,
    pub compute_stages: ComputeStages,
    pub render_stages: RenderStages,

    pub compute_init_bgs: BindGroups,
    pub compute_macros_bgs: BindGroups,
    pub compute_step_bgs: BindGroups,
    pub compute_tracers_bgs: BindGroups,

    pub render_density_bgs: BindGroups,
    pub render_speed_bgs: BindGroups,
    pub render_x_vel_bgs: BindGroups,
    pub render_y_vel_bgs: BindGroups,
    pub render_curl_bgs: BindGroups,
    pub render_tracers_bgs: BindGroups,
    pub render_flow_streams_bgs: BindGroups,
}

impl LBM {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        uniforms: Uniforms,
        num_tracers: u64,
    ) -> Self {
        let storage = wgpu::BufferUsages::STORAGE;
        let num_cells = uniforms.w * uniforms.h;
        let len_f = num_cells * 9;

        let uniforms_buffer = create_buffer_init(
            device,
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let fa_buffer = create_buffer(device, (len_f * 4) as u64, storage);
        let fb_buffer = create_buffer(device, (len_f * 4) as u64, storage);
        let types_buffer = create_buffer(device, (num_cells * 4) as u64, storage);
        let density_buffer = create_buffer(device, (num_cells * 4) as u64, storage);
        let velocity_buffer = create_buffer(device, (num_cells * 8) as u64, storage);

        let object_mask: Vec<u32> = (0..(uniforms.w_object * uniforms.h_object))
            .map(|index| {
                let x = (index % uniforms.w_object) as i32;
                let y = (index / uniforms.w_object) as i32;
                let cx = (uniforms.w_object / 2) as i32;
                let cy = (uniforms.h_object / 2) as i32;
                let radius = (uniforms.w_object.min(uniforms.h_object) / 2) as i32;
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius { 1u32 } else { 0u32 }
            })
            .collect();
        let object_buffer = create_buffer_init(
            device,
            bytemuck::cast_slice(&object_mask),
            storage,
        );
        let tracers_buffer = create_buffer(device, num_tracers * 16, storage);

        let buffers = LBMBuffers {
            uniforms: uniforms_buffer,
            fa: fa_buffer,
            fb: fb_buffer,
            types: types_buffer,
            density: density_buffer,
            veloctiy: velocity_buffer,
            object: object_buffer,
            tracers: tracers_buffer,
        };

        let compute_stages = ComputeStages {
            init: ComputeInitialConditions::new(device),
            macros: ComputeMacros::new(device),
            step: ComputeStep::new(device),
            tracers: ComputeTracers::new(device),
        };

        let render_stages = RenderStages {
            density: RenderDensity::new(device, config),
            speed: RenderSpeed::new(device, config),
            x_vel: RenderXVelocity::new(device, config),
            y_vel: RenderYVelocity::new(device, config),
            curl: RenderCurl::new(device, config),
            tracers: RenderTracers::new(device, config),
            flow_streams: RenderFlowStreams::new(device, config),
        };

        let compute_init_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &compute_stages.init.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &compute_stages.init.bgl1,
                &[
                    buffer_binding_entry(&buffers.fa, 0),
                    buffer_binding_entry(&buffers.fb, 1),
                    buffer_binding_entry(&buffers.types, 2),
                    buffer_binding_entry(&buffers.density, 3),
                    buffer_binding_entry(&buffers.veloctiy, 4),
                    buffer_binding_entry(&buffers.object, 5),
                    buffer_binding_entry(&buffers.tracers, 6),
                ],
            ),
        };

        let compute_macros_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &compute_stages.macros.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &compute_stages.macros.bgl1,
                &[
                    buffer_binding_entry(&buffers.fa, 0),
                    buffer_binding_entry(&buffers.density, 1),
                    buffer_binding_entry(&buffers.veloctiy, 2),
                ],
            ),
        };

        let compute_step_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &compute_stages.step.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &compute_stages.step.bgl1,
                &[
                    buffer_binding_entry(&buffers.fa, 0),
                    buffer_binding_entry(&buffers.fb, 1),
                    buffer_binding_entry(&buffers.types, 2),
                ],
            ),
        };

        let compute_tracers_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &compute_stages.tracers.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &compute_stages.tracers.bgl1,
                &[
                    buffer_binding_entry(&buffers.veloctiy, 0),
                    buffer_binding_entry(&buffers.tracers, 1),
                ],
            ),
        };

        let render_density_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.density.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.density.bgl1,
                &[
                    buffer_binding_entry(&buffers.density, 0),
                    buffer_binding_entry(&buffers.types, 1),
                ],
            ),
        };

        let render_speed_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.speed.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.speed.bgl1,
                &[
                    buffer_binding_entry(&buffers.veloctiy, 0),
                    buffer_binding_entry(&buffers.types, 1),
                ],
            ),
        };

        let render_x_vel_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.x_vel.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.x_vel.bgl1,
                &[
                    buffer_binding_entry(&buffers.veloctiy, 0),
                    buffer_binding_entry(&buffers.types, 1),
                ],
            ),
        };

        let render_y_vel_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.y_vel.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.y_vel.bgl1,
                &[
                    buffer_binding_entry(&buffers.veloctiy, 0),
                    buffer_binding_entry(&buffers.types, 1),
                ],
            ),
        };

        let render_curl_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.curl.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.curl.bgl1,
                &[
                    buffer_binding_entry(&buffers.veloctiy, 0),
                    buffer_binding_entry(&buffers.types, 1),
                ],
            ),
        };

        let render_tracers_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.tracers.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.tracers.bgl1,
                &[
                    buffer_binding_entry(&buffers.tracers, 0),
                ],
            ),
        };

        let render_flow_streams_bgs = BindGroups {
            bg0: create_bind_group(
                device,
                &render_stages.flow_streams.bgl0,
                &[buffer_binding_entry(&buffers.uniforms, 0)],
            ),
            bg1: create_bind_group(
                device,
                &render_stages.flow_streams.bgl1,
                &[
                    buffer_binding_entry(&buffers.veloctiy, 0),
                ],
            ),
        };

        Self {
            buffers,
            compute_stages,
            render_stages,
            compute_init_bgs,
            compute_macros_bgs,
            compute_step_bgs,
            compute_tracers_bgs,
            render_density_bgs,
            render_speed_bgs,
            render_x_vel_bgs,
            render_y_vel_bgs,
            render_curl_bgs,
            render_tracers_bgs,
            render_flow_streams_bgs,
        }
    }

    pub fn init(&self, encoder: &mut wgpu::CommandEncoder, wg_x: u32, wg_y: u32) {
        self.compute_stages.init.record(encoder, &self.compute_init_bgs.bg0, &self.compute_init_bgs.bg1, wg_x, wg_y);
    }

    pub fn compute_macros(&self, encoder: &mut wgpu::CommandEncoder, wg_x: u32, wg_y: u32) {
        self.compute_stages.macros.record(encoder, &self.compute_macros_bgs.bg0, &self.compute_macros_bgs.bg1, wg_x, wg_y);
    }

    pub fn step(&self, encoder: &mut wgpu::CommandEncoder, wg_x: u32, wg_y: u32, wg_tracers: u32, tracers_active: bool) {
        self.compute_stages.step.record(encoder, &self.compute_step_bgs.bg0, &self.compute_step_bgs.bg1, wg_x, wg_y);
        if tracers_active {
            self.compute_stages.tracers.record(encoder, &self.compute_tracers_bgs.bg0, &self.compute_tracers_bgs.bg1, wg_tracers);
        }
    }

    pub fn render_density(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let bgs = &self.render_density_bgs;

        self.render_stages
            .density
            .record(encoder, view, &bgs.bg0, &bgs.bg1);
    }

    pub fn render_speed(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let bgs = &self.render_speed_bgs;

        self.render_stages
            .speed
            .record(encoder, view, &bgs.bg0, &bgs.bg1);
    }

    pub fn render_x_velocty(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let bgs = &self.render_x_vel_bgs;

        self.render_stages
            .x_vel
            .record(encoder, view, &bgs.bg0, &bgs.bg1);
    }

    pub fn render_y_velocity(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let bgs = &self.render_y_vel_bgs;

        self.render_stages
            .y_vel
            .record(encoder, view, &bgs.bg0, &bgs.bg1);
    }

    pub fn render_curl(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let bgs = &self.render_curl_bgs;
        self.render_stages
            .curl
            .record(encoder, view, &bgs.bg0, &bgs.bg1);
    }

    pub fn render_tracers(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_vertices: u32,
    ) {
        let bgs = &self.render_tracers_bgs;
        self.render_stages
            .tracers
            .record(encoder, view, &bgs.bg0, &bgs.bg1, num_vertices);
    }

    pub fn render_flow_streams(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_vertices: u32,
    ) {
        let bgs = &self.render_flow_streams_bgs;
        self.render_stages
            .flow_streams
            .record(encoder, view, &bgs.bg0, &bgs.bg1, num_vertices);
    }

    pub fn upload_uniforms(&self, uniforms: Uniforms, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffers.uniforms, 0, bytemuck::cast_slice(&[uniforms]));
    }
}
