use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiState;

use crate::gpu::{gpu::GPU, resources::Uniforms};
use crate::lbm::LBM;

#[derive(PartialEq, Clone, Copy)]
pub enum Rendermode {
    Density,
    Speed,
    XVelocity,
    YVelocity,
    Curl,
}

pub struct AppState {
    pub gpu: GPU,
    pub lbm: LBM,
    pub uniforms: Uniforms,

    pub accumulated_time: f32,
    pub last_frame_time: Instant,
    pub paused: bool,
    pub render_mode: Rendermode,
    pub tracers_active: bool,
    pub flow_lines_active: bool,
    pub num_tracers: u32,

    pub re: f32,
    pub u_phys: f32,
    pub l_phys: f32,
    pub u_lb: f32,

    pub l_per_cell: f32,

    pub width: u32,
    pub height: u32,
    pub window_width: u32,
    pub window_height: u32,
    pub obj_width: u32,
    pub obj_height: u32,
    pub obj_x: u32,
    pub obj_y: u32,

    pub scale_min: f32,
    pub scale_max: f32,
}

impl AppState {
    fn preferred_sim_size(_window_width: u32, _window_height: u32) -> (u32, u32) {
        (960, 540)
    }

    pub fn new(gpu: GPU) -> Self {
        let window_width = gpu.config.width;
        let window_height = gpu.config.height;
        let (width, height) = Self::preferred_sim_size(window_width, window_height);
        let obj_width = 80;
        let obj_height = 80;
        let obj_x = (width * 25) / 100;
        let obj_y = (height - obj_height) / 2;
        let num_tracers = 600;

        let re = 550.0;
        let u_phys = 5.0;
        let l_phys = 2.8;
        let u_lb = 0.08;

        let l_per_cell = l_phys / (obj_height as f32);

        let nu_lb = (u_lb * obj_height as f32) / re;
        let tau = 3.0 * nu_lb + 0.5;

        let uniforms = Uniforms {
            w: width,
            h: height,
            tau,
            u_lb,
            w_object: obj_width,
            h_object: obj_height,
            is_first_step: 1,
            obj_x,
            obj_y,
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
        };

        let lbm = LBM::new(&gpu.device, &gpu.config, uniforms, num_tracers as u64);

        let max_speed = u_lb * 1.5;
        let (scale_min, scale_max) = (0.0_f32, max_speed);

        let app = Self {
            gpu,
            lbm,
            uniforms,
            accumulated_time: 0.0,
            last_frame_time: Instant::now(),
            paused: true,
            render_mode: Rendermode::Curl,
            tracers_active: false,
            flow_lines_active: false,
            num_tracers,
            re,
            u_phys,
            l_phys,
            u_lb,
            width,
            height,
            window_width,
            window_height,
            obj_width,
            obj_height,
            obj_x,
            obj_y,
            l_per_cell,
            scale_min,
            scale_max,
        };

        app.init_simulation();
        app
    }

    pub fn init_simulation(&self) {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Init Encoder") });

        let wg_x = self.width.div_ceil(8);
        let wg_y = self.height.div_ceil(8);
        self.lbm.init(&mut encoder, wg_x, wg_y);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn reset(&mut self) {
        self.accumulated_time = 0.0;
        self.last_frame_time = Instant::now();
        self.paused = false;

        let nu_lb = (self.u_lb * self.obj_height as f32) / self.re;
        self.uniforms.tau = 3.0 * nu_lb + 0.5;
        self.uniforms.w = self.width;
        self.uniforms.h = self.height;
        self.uniforms.w_object = self.obj_width;
        self.uniforms.h_object = self.obj_height;
        self.uniforms.obj_x = self.obj_x;
        self.uniforms.obj_y = self.obj_y;
        self.uniforms.is_first_step = 1;

        self.lbm = LBM::new(&self.gpu.device, &self.gpu.config, self.uniforms, self.num_tracers as u64);
        self.init_simulation();
    }

    pub fn resize_simulation(&mut self, sim_width: u32, sim_height: u32) {
        let sim_width = sim_width.max(64).min(1600);
        let sim_height = sim_height.max(64).min(1200);

        self.width = sim_width;
        self.height = sim_height;

        self.obj_width = self.obj_width.min(self.width.max(8));
        self.obj_height = self.obj_height.min(self.height.max(8));

        let nu_lb = (self.u_lb * self.obj_height as f32) / self.re;
        self.uniforms.tau = 3.0 * nu_lb + 0.5;
        self.uniforms.w = sim_width;
        self.uniforms.h = sim_height;
        self.uniforms.w_object = self.obj_width;
        self.uniforms.h_object = self.obj_height;
        self.uniforms.obj_x = self.obj_x;
        self.uniforms.obj_y = self.obj_y;
        self.uniforms.is_first_step = 1;

        self.lbm = LBM::new(&self.gpu.device, &self.gpu.config, self.uniforms, self.num_tracers as u64);
        self.init_simulation();
    }

    pub fn update_object_size(&mut self) {
        self.obj_width = self.obj_width.clamp(8, self.width.max(8));
        self.obj_height = self.obj_height.clamp(8, self.height.max(8));

        self.l_phys = (self.obj_height as f32) * self.l_per_cell;

        let nu_lb = (self.u_lb * self.obj_height as f32) / self.re;
        self.uniforms.tau = 3.0 * nu_lb + 0.5;
        self.uniforms.w = self.width;
        self.uniforms.h = self.height;
        self.uniforms.w_object = self.obj_width;
        self.uniforms.h_object = self.obj_height;
        self.uniforms.obj_x = self.obj_x;
        self.uniforms.obj_y = self.obj_y;
        self.uniforms.is_first_step = 1;

        self.lbm = LBM::new(&self.gpu.device, &self.gpu.config, self.uniforms, self.num_tracers as u64);
        self.init_simulation();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.window_width = new_size.width;
        self.window_height = new_size.height;
        self.gpu.config.width = new_size.width;
        self.gpu.config.height = new_size.height;
        self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);
    }

    pub fn color_scale_info(&self) -> (&'static str, f32, f32) {
        let label = match self.render_mode {
            Rendermode::Density => "Density",
            Rendermode::Speed => "Speed",
            Rendermode::XVelocity => "X-Velocity",
            Rendermode::YVelocity => "Y-Velocity",
            Rendermode::Curl => "Curl",
        };
        (label, self.scale_min, self.scale_max)
    }

    pub fn auto_set_scale(&mut self) {
        let max_speed = self.u_lb * 1.5;
        match self.render_mode {
            Rendermode::Density => {
                self.scale_min = 0.98;
                self.scale_max = 1.02;
            }
            Rendermode::Speed => {
                self.scale_min = 0.0;
                self.scale_max = max_speed;
            }
            Rendermode::XVelocity | Rendermode::YVelocity => {
                self.scale_min = -max_speed;
                self.scale_max = max_speed;
            }
            Rendermode::Curl => {
                let s = self.u_lb * 0.15;
                self.scale_min = -s;
                self.scale_max = s;
            }
        }
    }

    pub fn update_physics(&mut self) {
        let now = Instant::now();

        let dt = now.duration_since(self.last_frame_time).as_secs_f32().min(1.0 / 30.0);
        self.last_frame_time = now;

        if self.paused {
            return;
        }

        self.accumulated_time += dt;

        let nu_lb = (self.u_lb * self.obj_height as f32) / self.re;
        self.uniforms.tau = 3.0 * nu_lb + 0.5;
        let c_t = (self.l_phys * self.u_lb) / (self.obj_height as f32 * self.u_phys);

        let mut steps = (self.accumulated_time / c_t).floor() as u32;

        const MAX_STEPS_PER_FRAME: u32 = 8;
        if steps > MAX_STEPS_PER_FRAME {
            steps = MAX_STEPS_PER_FRAME;
            self.accumulated_time = 0.0;
        } else {
            self.accumulated_time -= (steps as f32) * c_t;
        }

        if steps == 0 {
            return;
        }

        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Compute Encoder") });
        let wg_x = self.width.div_ceil(8);
        let wg_y = self.height.div_ceil(8);
        let wg_tracers = self.num_tracers.div_ceil(64);

        for i in 0..steps {
            self.uniforms.is_first_step = if i == 0 { 1 } else { 0 };
            self.lbm.upload_uniforms(self.uniforms, &self.gpu.queue);

            self.lbm.step(&mut encoder, wg_x, wg_y, wg_tracers, self.tracers_active);
            self.lbm.compute_macros(&mut encoder, wg_x, wg_y);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
    }
}

struct LbmAppRunner {
    app_state: Option<AppState>,
    egui_state: Option<EguiState>,
    egui_renderer: Option<EguiRenderer>,
    egui_ctx: egui::Context,
    window: Option<Arc<Window>>,
}

impl LbmAppRunner {
    fn new() -> Self {
        Self {
            app_state: None,
            egui_state: None,
            egui_renderer: None,
            egui_ctx: egui::Context::default(),
            window: None,
        }
    }
}

impl ApplicationHandler for LbmAppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {

            let window_attr = Window::default_attributes()
                .with_title("LBM WebGPU")
                .with_maximized(true);
            let window = Arc::new(event_loop.create_window(window_attr).expect("Failed to create window"));
            self.window = Some(window.clone());

            let gpu = pollster::block_on(GPU::new(window.clone()));
            let app = AppState::new(gpu);

            let egui_state = EguiState::new(
                self.egui_ctx.clone(),
                egui::ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                None,
                None,
            );

            let egui_renderer = EguiRenderer::new(
                &app.gpu.device,
                app.gpu.config.format,
                egui_wgpu::RendererOptions::default(),

            );

            self.app_state = Some(app);
            self.egui_state = Some(egui_state);
            self.egui_renderer = Some(egui_renderer);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else { return };
        let Some(app) = &mut self.app_state else { return };
        let Some(egui_state) = &mut self.egui_state else { return };

        let egui_handled = egui_state.on_window_event(window, &event);

        if !egui_handled.consumed {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {

                    app.update_physics();

                    let mut style = self.egui_ctx.style_of(egui::Theme::Light).as_ref().clone();
                    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
                    style.spacing.window_margin = egui::Margin::same(8);
                    style.spacing.button_padding = egui::vec2(7.0, 4.0);
                    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(24, 27, 38, 220);
                    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(220, 228, 245);
                    style.visuals.window_fill = egui::Color32::from_rgba_unmultiplied(17, 21, 28, 220);
                    style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(90, 110, 160, 180));
                    self.egui_ctx.set_style_of(egui::Theme::Light, style);

                    let raw_input = egui_state.take_egui_input(window);
                    let mut full_output = self.egui_ctx.run_ui(raw_input, |ctx| {

                        egui::Window::new("Simulation Controls")
                            .default_width(300.0)
                            .min_width(250.0)
                            .max_width(320.0)
                            .resizable(false)
                            .frame(egui::Frame::new().inner_margin(egui::Margin::same(10)).fill(egui::Color32::from_rgba_unmultiplied(18, 22, 31, 220)).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(90, 110, 160, 180))))
                            .show(ctx, |ui| {
                            ui.set_min_height(0.0);
                            ui.vertical(|ui| {
                                ui.label("Physics");
                                ui.add(egui::Slider::new(&mut app.re, 50.0..=2000.0).text("Re").fixed_decimals(0));
                                ui.add(egui::Slider::new(&mut app.u_phys, 0.1..=5.0).text("U").fixed_decimals(2));

                                let l_resp = ui.add(egui::Slider::new(&mut app.l_phys, 0.1..=20.0).text("L (scale)").fixed_decimals(2));
                                if l_resp.changed() {

                                    let new_h = (app.l_phys / app.l_per_cell).round() as i32;
                                    let new_w = ((app.obj_width as f32) * (new_h as f32) / (app.obj_height as f32)).round() as i32;
                                    app.obj_height = new_h.max(8).min(app.height as i32) as u32;
                                    app.obj_width = new_w.max(8).min(app.width as i32) as u32;
                                    app.update_object_size();
                                }

                                ui.add(egui::Slider::new(&mut app.u_lb, 0.01..=0.25).text("uLB").fixed_decimals(3));

                                ui.separator();
                                ui.label("Resolution");
                                if ui.add(egui::Slider::new(&mut app.width, 64..=1200).text("W").fixed_decimals(0)).changed() {
                                    app.resize_simulation(app.width, app.height);
                                }
                                if ui.add(egui::Slider::new(&mut app.height, 64..=900).text("H").fixed_decimals(0)).changed() {
                                    app.resize_simulation(app.width, app.height);
                                }

                                ui.separator();
                                ui.label("Object Size");
                                let max_box_w = app.width.max(8);
                                let max_box_h = app.height.max(8);
                                app.obj_width = app.obj_width.min(max_box_w);
                                app.obj_height = app.obj_height.min(max_box_h);
                                if ui.add(egui::Slider::new(&mut app.obj_width, 8..=max_box_w).text("Box W").fixed_decimals(0)).changed() {
                                    app.update_object_size();
                                }
                                if ui.add(egui::Slider::new(&mut app.obj_height, 8..=max_box_h).text("Box H").fixed_decimals(0)).changed() {
                                    app.update_object_size();
                                }

                                let max_x = app.width.saturating_sub(app.obj_width);
                                let max_y = app.height.saturating_sub(app.obj_height);
                                if ui.add(egui::Slider::new(&mut app.obj_x, 0..=max_x).text("Pos X").fixed_decimals(0)).changed() {
                                    app.update_object_size();
                                }
                                if ui.add(egui::Slider::new(&mut app.obj_y, 0..=max_y).text("Pos Y").fixed_decimals(0)).changed() {
                                    app.update_object_size();
                                }

                                ui.separator();
                                ui.label("View");
                                egui::ComboBox::from_label("Mode")
                                    .selected_text(match app.render_mode {
                                        Rendermode::Speed => "Speed",
                                        Rendermode::Curl => "Curl",
                                        Rendermode::Density => "Density",
                                        Rendermode::XVelocity => "X-Vel",
                                        Rendermode::YVelocity => "Y-Vel",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut app.render_mode, Rendermode::Speed, "Speed");
                                        ui.selectable_value(&mut app.render_mode, Rendermode::Curl, "Curl");
                                        ui.selectable_value(&mut app.render_mode, Rendermode::Density, "Density");
                                        ui.selectable_value(&mut app.render_mode, Rendermode::XVelocity, "X-Vel");
                                        ui.selectable_value(&mut app.render_mode, Rendermode::YVelocity, "Y-Vel");
                                    });

                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut app.tracers_active, "Tracers");
                                    ui.checkbox(&mut app.flow_lines_active, "Flow Lines");
                                    if ui.button("Reset").clicked() {
                                        app.reset();
                                    }
                                    if ui.button(if app.paused { "Play" } else { "Pause" }).clicked() {
                                        app.paused = !app.paused;
                                    }
                                });
                            });
                        });
                    });

                    let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [app.gpu.config.width, app.gpu.config.height],
                        pixels_per_point: window.scale_factor() as f32,
                    };

                    let mut encoder = app.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

                    if let Some(egui_renderer) = &mut self.egui_renderer {
                        for (id, image_delta) in &full_output.textures_delta.set {
                            egui_renderer.update_texture(&app.gpu.device, &app.gpu.queue, *id, &image_delta[0]);
                        }

                        egui_renderer.update_buffers(
                            &app.gpu.device,
                            &app.gpu.queue,
                            &mut encoder,
                            &paint_jobs,
                            &screen_descriptor,
                        );
                    }

                    let output = match app.gpu.surface.get_current_texture() {
                        wgpu::CurrentSurfaceTexture::Success(texture) => texture,
                        _ => {
                            if let Some(egui_renderer) = &mut self.egui_renderer {
                                for id in &full_output.textures_delta.free {
                                    egui_renderer.free_texture(id);
                                }
                            }
                            full_output.textures_delta.clear();
                            return;
                        }
                    };

                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    match app.render_mode {
                        Rendermode::Density => app.lbm.render_density(&mut encoder, &view),
                        Rendermode::Speed => app.lbm.render_speed(&mut encoder, &view),
                        Rendermode::XVelocity => app.lbm.render_x_velocty(&mut encoder, &view),
                        Rendermode::YVelocity => app.lbm.render_y_velocity(&mut encoder, &view),
                        Rendermode::Curl => app.lbm.render_curl(&mut encoder, &view),
                    }

                    if app.flow_lines_active {
                        let stream_step = 4_u32;
                        let stream_count = app.width.div_ceil(stream_step) * app.height.div_ceil(stream_step);
                        app.lbm.render_flow_streams(&mut encoder, &view, stream_count);
                    }

                    if app.tracers_active {
                        app.lbm.render_tracers(&mut encoder, &view, app.num_tracers);
                    }

                    let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })];

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Egui Render Pass"),
                            color_attachments: &color_attachments,
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        }).forget_lifetime();

                        if let Some(egui_renderer) = &self.egui_renderer {
                            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        }
                    }

                    app.gpu.queue.submit(std::iter::once(encoder.finish()));
                    app.gpu.queue.present(output);

                    if let Some(egui_renderer) = &mut self.egui_renderer {
                        for id in &full_output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }
                    }
                    full_output.textures_delta.clear()
                }
                _ => {}
            }
        }
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().expect("Failed to create winit event loop");
    let mut runner = LbmAppRunner::new();
    event_loop.run_app(&mut runner).expect("Failed to run event loop");
}