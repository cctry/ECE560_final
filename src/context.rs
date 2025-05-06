use std::time::Duration;

use crate::render::Camera;
use crate::render::Renderable;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::keyboard::PhysicalKey;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};
pub struct ContextState {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub new_terrain: bool,
    pub cursor_captured: bool,
}

fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Perlin Depth Texture"),
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub struct Context<'a> {
    context_data: ContextState,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: &'a Window,
    pipelines: Vec<Box<dyn Renderable>>,
    depth_texture_view: wgpu::TextureView,
    pub camera: Camera,
}

impl<'a> Context<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Context<'a> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width,
            height: height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let camera = Camera::new(&device, config.width, config.height);

        let depth_texture_view = create_depth_texture(&device, width, height);

        Context {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            context_data: ContextState {
                size: PhysicalSize::new(width, height),
                new_terrain: true,
                cursor_captured: false,
            },
            window,
            pipelines: Vec::new(),
            camera,
            depth_texture_view,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        let mut size = match new_size {
            Some(size) => size,
            None => *self.size(),
        };

        size.width = size.width.min(1280);
        size.height = size.height.min(720);
        if size.width > 0 && size.height > 0 {
            *self.size() = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.resize(size.width, size.height);
            self.depth_texture_view = create_depth_texture(&self.device, size.width, size.height);
        }
        log::info!("Surface resize to {0:?}", self.size());
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let mut res = match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(code),
                        ..
                    },
                ..
            } => self.camera.process_key(code),
            _ => false,
        };
        for pass in &mut self.pipelines {
            res |= pass.input(event, &self.context_data);
        }
        res
    }

    pub fn update(&mut self, dt: &Duration) {
        self.camera.update(dt, &self.queue);
        for pass in &mut self.pipelines {
            pass.update(&mut self.context_data, &self.queue);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Perlin Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for pipeline in &mut self.pipelines {
                pipeline.render(&mut render_pass, &self.camera);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn add_render_pass<T: Renderable + 'static>(&mut self) {
        let pass = T::new(&self.device, &self.config, &self.camera);
        self.pipelines.push(Box::new(pass));
    }

    pub fn size(&mut self) -> &mut PhysicalSize<u32> {
        &mut self.context_data.size
    }

    pub fn toggle_cursor_capture(&mut self) -> bool {
        self.context_data.cursor_captured = !self.context_data.cursor_captured;

        // Set cursor grab mode
        if let Err(e) = self
            .window
            .set_cursor_grab(if self.context_data.cursor_captured {
                winit::window::CursorGrabMode::Confined
            } else {
                winit::window::CursorGrabMode::None
            })
        {
            log::warn!("Failed to set cursor grab: {:?}", e);
        }

        // Set cursor visibility
        self.window
            .set_cursor_visible(!self.context_data.cursor_captured);
        true
    }

    pub fn is_cursor_captured(&self) -> bool {
        self.context_data.cursor_captured
    }
}
