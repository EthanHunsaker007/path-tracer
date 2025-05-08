mod app;
mod config;
mod input;

use app::*;
use config::*;
use input::*;
use std::{
    time::{Duration, Instant},
    vec,
};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FrameUniform {
    global_frame_info: [u32; 4],
}

struct FrameInfo {
    last_frame: Instant,
    frame_accum: Duration,
    frame_count: u32,
    last_fps: Instant,
    frame_uniform: FrameUniform,
    frame_buffer: Option<wgpu::Buffer>,
}

struct SurfaceState<'a> {
    window: &'a Window,
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    frame_info: FrameInfo,
}

impl<'a> SurfaceState<'a> {
    async fn new(window: &'a Window) -> SurfaceState<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            //            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let frame_uniform = FrameUniform {
            global_frame_info: [0; 4],
        };

        let frame_info = FrameInfo {
            last_frame: Instant::now(),
            frame_accum: Duration::ZERO,
            frame_count: 0,
            frame_uniform,
            frame_buffer: None,
            last_fps: Instant::now(),
        };

        SurfaceState {
            window,
            surface,
            adapter,
            config,
            size,
            frame_info,
        }
    }

    fn configure_surface(&mut self, device: &wgpu::Device) {
        self.surface.configure(&device, &self.config);
        self.frame_info.frame_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Frame Buffer"),
                contents: bytemuck::cast_slice(&[self.frame_info.frame_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));
    }

    fn update_frame_buffer(&self, queue: &wgpu::Queue) {
        if let Some(buffer) = &self.frame_info.frame_buffer {
            queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[self.frame_info.frame_uniform]),
            );
        }
    }

    fn toggle_fullscreen(&mut self) {
        if self.window.fullscreen().is_some() {
            self.window.set_fullscreen(None);
        } else {
            self.window
                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }
    }
}

struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    sampler: wgpu::Sampler,
}

impl GpuContext {
    async fn new(adapter: &wgpu::Adapter) -> GpuContext {
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        GpuContext {
            device,
            queue,
            sampler,
        }
    }
}


struct State<'a> {
    surface_state: SurfaceState<'a>,
    gpu_context: GpuContext,
    camera: Camera,
    scene: Scene,
    config: StateConfigs,
    bind_groups: BindGroups,
    textures: Textures,
    pipelines: Pipelines,
    input_handler: InputHandler,
    timestep: Duration,
    quit_flag: bool,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let quit_flag = false;
        let config = StateConfigs::default();
        let mut surface_state = SurfaceState::new(window).await;
        let gpu_context = GpuContext::new(&surface_state.adapter).await;
        surface_state.configure_surface(&gpu_context.device);
        let camera = Camera::new(
            &surface_state.size,
            &gpu_context.device,
            &gpu_context.queue,
            config.fov,
        );
        let mut scene = scene::Scene::new(&gpu_context.device);
        scene.setup_test_scene(100, &gpu_context.device);
        //        scene.add_sphere(2000.0, [0.5; 3], [0.0, 2000.0, 0.0]);
        //        scene.add_sphere(1.0, [0.5; 3], [0.0, -1.0, 0.0]);
        //        scene.update_sphere_buffer(&gpu_context.device);
        let textures = Textures::new(&gpu_context.device, &surface_state.size);
        let bind_groups = BindGroups::new(
            &gpu_context.device,
            &gpu_context.sampler,
            &scene.sphere_buffer,
            &scene.material_buffer,
            &camera.buffer,
            &surface_state.frame_info.frame_buffer,
            &textures.texture_buffer_a,
            &textures.texture_buffer_b,
            &textures.surface_texture_view,
        );
        let pipelines = Pipelines::new(&gpu_context.device, &surface_state.config, &bind_groups);
        let input_handler = InputHandler::new_defaults();

        Self {
            surface_state,
            gpu_context,
            camera,
            scene,
            config,
            bind_groups,
            textures,
            pipelines,
            input_handler,
            timestep: Duration::from_secs_f32(1.0 / 120.0),
            quit_flag,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_state.size = new_size;
            self.surface_state.config.width = new_size.width;
            self.surface_state.config.height = new_size.height;
            self.surface_state
                .surface
                .configure(&self.gpu_context.device, &self.surface_state.config);

            self.camera.camera.resize(new_size);
            self.camera.build_uniform();
            self.camera.update_buffer(&self.gpu_context.queue);
            self.rebuild_texture_and_bind_groups(&new_size);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.gpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipelines.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_groups.compute_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.bind_groups.camera_bind_group, &[]);
            compute_pass.set_bind_group(2, &self.bind_groups.scene_bind_group, &[]);
            if self
                .surface_state
                .frame_info
                .frame_uniform
                .global_frame_info[0]
                % 2
                == 0
            {
                compute_pass.set_bind_group(3, &self.bind_groups.bind_group_a_read_b_write, &[]);
            } else {
                compute_pass.set_bind_group(3, &self.bind_groups.bind_group_b_read_a_write, &[])
            }

            let (w, h) = (
                self.surface_state.size.width,
                self.surface_state.size.height,
            );
            compute_pass.dispatch_workgroups((w + 7) / 8, (h + 7) / 8, 1);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipelines.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_groups.fragment_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.gpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn rebuild_texture_and_bind_groups(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        self.textures = Textures::new(&self.gpu_context.device, new_size);
        self.bind_groups.rebuild_compute_bind_group(
            &self.gpu_context.device,
            &self.surface_state.frame_info.frame_buffer,
            &self.textures.surface_texture_view,
        );
        self.bind_groups.rebuild_fragment_bind_group(
            &self.gpu_context.device,
            &self.gpu_context.sampler,
            &self.textures.surface_texture_view,
        );
        self.bind_groups.rebuild_texture_buffer_bind_groups(
            &self.gpu_context.device,
            &self.textures.texture_buffer_a,
            &self.textures.texture_buffer_b,
        );
    }

    fn quit(&mut self) {
        self.quit_flag = true;
    }

    fn get_ticks(&mut self) -> u32 {
        let mut ticks = 0;
        let now = Instant::now();
        let frame_length = now - self.surface_state.frame_info.last_frame;
        self.surface_state.frame_info.last_frame = now;

        let frame_length = frame_length.min(Duration::from_millis(250));
        self.surface_state.frame_info.frame_accum += frame_length;

        while self.surface_state.frame_info.frame_accum >= self.timestep {
            ticks += 1;
            self.surface_state.frame_info.frame_accum -= self.timestep;
        }

        self.surface_state.frame_info.frame_count += 1;
        if now.duration_since(self.surface_state.frame_info.last_fps) >= Duration::from_secs(1) {
            self.surface_state.frame_info.last_fps = now;
            println!("The FPS is: {}", self.surface_state.frame_info.frame_count);
            self.surface_state.frame_info.frame_count = 0;
        }
        ticks
    }

    fn process_event(
        &mut self,
        event: winit::event::Event<()>,
        control_flow: &winit::event_loop::EventLoopWindowTarget<()>,
    ) {
        match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested {} => control_flow.exit(),

                WindowEvent::Resized(physical_size) => self.resize(*physical_size),

                WindowEvent::RedrawRequested => match self.render() {
                    Ok(_) => {}

                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.resize(self.surface_state.size)
                    }

                    Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                        log::error!("OutOfMemory");
                        control_flow.exit();
                    }

                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    let mut dispatcher = ActionDispatcher::new();

    window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();

    window.set_cursor_visible(false);

    event_loop
        .run(move |event, control_flow| {
            control_flow.set_control_flow(ControlFlow::Poll);

            if state.quit_flag == true {
                control_flow.exit()
            }

            state
                .input_handler
                .process_input(&event, &mut state.camera, &state.gpu_context.queue);

            if let Event::NewEvents(StartCause::Poll) = event {
                state
                    .surface_state
                    .frame_info
                    .frame_uniform
                    .global_frame_info[0] += 1;
                if !state.input_handler.flags.camera_has_moved {
                    state
                        .surface_state
                        .frame_info
                        .frame_uniform
                        .global_frame_info[1] += 1;
                } else {
                    state
                        .surface_state
                        .frame_info
                        .frame_uniform
                        .global_frame_info[1] = 0;
                }
                state
                    .surface_state
                    .update_frame_buffer(&state.gpu_context.queue);

                let mut logic_ticks = state.get_ticks();
                while logic_ticks > 0 {
                    state.input_handler.flags.camera_has_moved = false;
                    let actions = state.input_handler.get_actions();
                    dispatcher.dispatch(actions, &mut state);

                    logic_ticks -= 1
                }
                state.surface_state.window.request_redraw();
            };

            state.process_event(event, control_flow);
        })
        .unwrap();
}
