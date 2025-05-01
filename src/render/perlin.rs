use super::renderable::Renderable;
use crate::context::ContextState;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use web_sys::js_sys::Math;
use winit::event::WindowEvent;

const SIZE: usize = 512;

fn generate_heightmap() -> Vec<f32> {
    let seed = Math::random() as u32;
    let fbm: Fbm<Perlin> = Fbm::new(seed)
        .set_lacunarity(2.0)
        .set_persistence(0.5)
        .set_octaves(6);

    let scale = 10.0; // Adjust scale to control the frequency of the noise
    let mut heightmap = vec![0.0; SIZE * SIZE];
    for z in 0..SIZE {
        for x in 0..SIZE {
            let val = fbm.get([
                x as f64 * scale / SIZE as f64,
                z as f64 * scale / SIZE as f64,
            ]) as f32;
            let normalized = (val + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
            heightmap[z * SIZE + x] = normalized;
        }
    }

    heightmap
}

pub struct PerlinPass {
    new_terrian: bool,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    noise_texture: wgpu::Texture,
    _noise_texture_view: wgpu::TextureView,
}

impl Renderable for PerlinPass {
    fn input(&mut self, _event: &WindowEvent, _context: &ContextState) -> bool {
        false
    }

    fn update(&mut self, context: &mut ContextState, queue: &wgpu::Queue) {
        self.new_terrian = context.new_terrian;
        if self.new_terrian {
            let heightmap = generate_heightmap();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.noise_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&heightmap),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(SIZE as u32 * std::mem::size_of::<f32>() as u32),
                    rows_per_image: Some(SIZE as u32),
                },
                wgpu::Extent3d {
                    width: SIZE as u32,
                    height: SIZE as u32,
                    depth_or_array_layers: 1,
                },
            );
        }
        context.new_terrian = false;
    }

    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        // Create the texture for the heightmap
        let noise_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Noise Texture"),
            size: wgpu::Extent3d {
                width: SIZE as u32,
                height: SIZE as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let noise_texture_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Render pipeline
        let render_shader = device.create_shader_module(wgpu::include_wgsl!("perlin_render.wgsl"));
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Perlin Render Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Perlin Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&noise_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Perlin Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        Self {
            new_terrian: true,
            render_bind_group,
            render_pipeline,
            noise_texture,
            _noise_texture_view: noise_texture_view,
        }
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
}
