use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::render::Renderable;
pub struct ContextState {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub new_terrian: bool,
}

pub struct Context<'a> {
    context_data: ContextState,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: &'a Window,
    render_passes: Vec<Box<dyn Renderable>>,
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

        Context {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            context_data: ContextState {
                size: PhysicalSize::new(width, height),
                new_terrian: true,
            },
            window,
            render_passes: Vec::new(),
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

        size.width = size.width.min(800);
        size.height = size.height.min(600);
        if size.width > 0 && size.height > 0 {
            *self.size() = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
        log::info!("Surface resize to {0:?}", self.size());
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let mut res = false;
        for pass in &mut self.render_passes {
            res |= pass.input(event, &self.context_data);
        }
        res
    }

    pub fn update(&mut self) {
        for pass in &mut self.render_passes {
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

        for pass in &mut self.render_passes {
            pass.render(&mut encoder, &view, &self.device, &self.queue);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn add_render_pass<T: Renderable + 'static>(&mut self) {
        let pass = T::new(&self.device, &self.config);
        self.render_passes.push(Box::new(pass));
    }

    pub fn size(&mut self) -> &mut PhysicalSize<u32> {
        &mut self.context_data.size
    }
}
