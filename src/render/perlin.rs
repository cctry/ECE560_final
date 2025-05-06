use std::u32;

use super::{Camera, renderable::Renderable};
use crate::context::ContextState;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use web_sys::js_sys::Math;
use wgpu::BufferUsages;
use winit::event::WindowEvent;

const SIZE: usize = 512;
const SCALE: f32 = 32.0;
const SHIFT: f32 = 12.0;
const LACUNARITY: f64 = 2.0;
const PERSISTENCE: f64 = 0.5;
const OCTAVES: usize = 6;
const NOISE_SCALE: f64 = 10.0;

fn generate_heightmap() -> Vec<f32> {
    let seed = Math::random() as u32;
    let fbm: Fbm<Perlin> = Fbm::new(seed)
        .set_lacunarity(LACUNARITY)
        .set_persistence(PERSISTENCE)
        .set_octaves(OCTAVES);

    let mut heightmap = vec![0.0; SIZE * SIZE];
    for z in 0..SIZE {
        for x in 0..SIZE {
            let val = fbm.get([
                x as f64 * NOISE_SCALE / SIZE as f64,
                z as f64 * NOISE_SCALE / SIZE as f64,
            ]) as f32;
            let normalized = (val + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
            heightmap[z * SIZE + x] = normalized;
        }
    }

    heightmap
}

fn tessellation(height_map: &Vec<f32>) -> (Vec<f32>, Vec<u32>) {
    // vertex generation
    let mut vertices = Vec::with_capacity(SIZE * SIZE * 3);
    for i in 0..SIZE {
        for j in 0..SIZE {
            let h = height_map[i * SIZE + j];
            vertices.push(-(SIZE as f32) / 2.0 + i as f32); // v.x
            vertices.push(h * SCALE - SHIFT); // v.y
            vertices.push(-(SIZE as f32) / 2.0 + j as f32); // v.z
        }
    }
    // indices generation
    let mut indices = Vec::with_capacity((SIZE - 1) * SIZE * 2 + SIZE - 2);
    for i in 0..SIZE - 1 {
        if i % 2 == 0 {
            // Even bottom→top
            for j in 0..SIZE {
                // bottom
                indices.push(((i + 1) * SIZE + j) as u32);
                // top
                indices.push((i * SIZE + j) as u32);
            }
        } else {
            // Odd top→bottom
            for j in (0..SIZE).rev() {
                // top
                indices.push((i * SIZE + j) as u32);
                // bottom
                indices.push(((i + 1) * SIZE + j) as u32);
            }
        }

        // primitive‑restart
        if i < SIZE - 2 {
            indices.push(u32::MAX);
        }
    }

    (vertices, indices)
}

pub struct PerlinPass {
    new_terrain: bool,
    render_pipeline: wgpu::RenderPipeline,
    terrain_index_buffer: wgpu::Buffer,
    terrain_vertex_buffer: wgpu::Buffer,
}

impl Renderable for PerlinPass {
    fn input(&mut self, _event: &WindowEvent, _context: &ContextState) -> bool {
        false
    }

    fn update(&mut self, context: &mut ContextState, queue: &wgpu::Queue) {
        self.new_terrain = context.new_terrain;
        if self.new_terrain {
            let heightmap = generate_heightmap();
            let (vertices, indices) = tessellation(&heightmap);
            queue.write_buffer(
                &self.terrain_index_buffer,
                0,
                bytemuck::cast_slice(&indices),
            );
            queue.write_buffer(
                &self.terrain_vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices),
            );
        }
        context.new_terrain = false;
    }

    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera: &Camera) -> Self {
        let terrain_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Perlin Index Buffer"),
            size: ((SIZE * (SIZE - 1) * 2 + SIZE - 2) * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        let terrain_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Perlin Vertex Buffer"),
            size: (SIZE * SIZE * 3 * std::mem::size_of::<f32>()) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("terrain.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Perlin Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Perlin Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 3 * std::mem::size_of::<f32>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        Self {
            new_terrain: true,
            render_pipeline,
            terrain_index_buffer,
            terrain_vertex_buffer,
        }
    }

    fn render(&mut self, pass: &mut wgpu::RenderPass, camera: &Camera) {
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &camera.bind_group, &[]);
        pass.set_vertex_buffer(0, self.terrain_vertex_buffer.slice(..));
        pass.set_index_buffer(
            self.terrain_index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        let index_count = (SIZE - 1) * SIZE * 2 + (SIZE - 2); // Or store this when generating indices
        pass.draw_indexed(0..index_count as u32, 0, 0..1);
    }
}
