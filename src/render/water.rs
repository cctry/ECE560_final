use wgpu::util::DeviceExt;

use super::Renderable;

const SIZE: usize = 256;
const SIZE_F32: f32 = SIZE as f32;

const WATER_LEVEL: f32 = 0.0;

#[rustfmt::skip]
const WATER_VERTICES:&[[f32; 3]] = &[
    //   x,     y,     z
    [-SIZE_F32, WATER_LEVEL, -SIZE_F32],
    [ SIZE_F32, WATER_LEVEL, -SIZE_F32],
    [ SIZE_F32, WATER_LEVEL,  SIZE_F32],
    [-SIZE_F32, WATER_LEVEL,  SIZE_F32],
];

#[rustfmt::skip]
const WATER_INDICES: &[u32] = &[
    0, 1, 2,
    0, 2, 3,
];

pub struct WaterPass {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Renderable for WaterPass {
    fn render(&mut self, pass: &mut wgpu::RenderPass, camera: &super::Camera) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &camera.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..WATER_INDICES.len() as u32, 0, 0..1);
    }

    fn input(
        &mut self,
        _event: &winit::event::WindowEvent,
        _context: &crate::context::ContextState,
    ) -> bool {
        false
    }

    fn update(&mut self, _context: &mut crate::context::ContextState, _queue: &wgpu::Queue) {}

    fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera: &super::Camera,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Vertex Buffer"),
            contents: bytemuck::cast_slice(WATER_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Index Buffer"),
            contents: bytemuck::cast_slice(WATER_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Water Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("water.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Water Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: Default::default(),
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}
