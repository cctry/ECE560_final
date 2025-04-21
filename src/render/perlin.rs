use super::renderable::Renderable;
use crate::context::ContextState;
use web_sys::js_sys::Math;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;

#[repr(C)]
#[repr(align(16))]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Parameters {
    persistence: f32,
    octaves: u32,
    lacunarity: f32,
    scale: u32, // number of grid points
    p_table: [[u32; 4]; 128],
}

pub struct PerlinPass {
    parameters: Parameters,
    parameter_buffer: wgpu::Buffer,
    new_terrian: bool,
    render_pipeline: wgpu::RenderPipeline,
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::RenderPipeline,
    _noise_texture: wgpu::Texture,
    noise_texture_view: wgpu::TextureView,
}

fn shuffle<T>(arr: &mut [T]) {
    let len = arr.len();
    for i in (1..len).rev() {
        let j = (Math::random() * (i + 1) as f64) as usize;
        arr.swap(i, j);
    }
}

fn new_ptable() -> [[u32; 4]; 128] {
    let mut p_table: [u32; 256] = std::array::from_fn(|i| i as u32);
    shuffle(&mut p_table);
    // Duplicate the table
    let mut p_table_full = [0; 512];
    p_table_full[..256].copy_from_slice(&p_table);
    p_table_full[256..].copy_from_slice(&p_table);
    let mut result = [[0u32; 4]; 128];
    for i in 0..128 {
        for j in 0..4 {
            result[i][j] = p_table_full[i * 4 + j];
        }
    }
    result
}

impl Renderable for PerlinPass {
    fn input(&mut self, _event: &WindowEvent, _context: &ContextState) -> bool {
        false
    }

    fn update(&mut self, context: &mut ContextState, queue: &wgpu::Queue) {
        self.new_terrian = context.new_terrian;
        if self.new_terrian {
            let p_table = new_ptable();
            self.parameters.p_table.copy_from_slice(&p_table);
            queue.write_buffer(
                &self.parameter_buffer,
                0,
                bytemuck::cast_slice(&[self.parameters]),
            );
        }
        context.new_terrian = false;
    }

    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let parameters = Parameters {
            persistence: 0.5,
            octaves: 6,
            lacunarity: 2.0,
            scale: 10,
            p_table: new_ptable(),
        };

        // Create a buffer for the parameters
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PerlinPass Uniform Buffer"),
            contents: bytemuck::cast_slice(&[parameters]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the texture as output
        let noise_width = 512;
        let noise_height = 512;
        let noise_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Noise Texture"),
            size: wgpu::Extent3d {
                width: noise_width,
                height: noise_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let noise_texture_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("PerlinPass Bind Group Layout"),
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PerlinPass Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("perlin_compute.wgsl"));

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Perlin Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Perlin Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &compute_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &compute_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
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
            parameters,
            compute_bind_group,
            compute_pipeline,
            render_bind_group,
            render_pipeline,
            _noise_texture: noise_texture,
            noise_texture_view,
            new_terrian: true,
            parameter_buffer: uniform_buffer,
        }
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        // Compute pass
        if self.new_terrian {
            let mut compute_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Compute Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.noise_texture_view,
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
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.draw(0..6, 0..1);
            self.new_terrian = false;
        }

        // Render pass
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
