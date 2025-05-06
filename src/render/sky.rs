use wgpu::util::DeviceExt;

use super::Renderable;

#[rustfmt::skip]
const SKYBOX_VERTICES: &[[f32; 3]] = &[
    // +X (right) face, inward normal (‑1,0,0)
    [ 1.0,  1.0, -1.0], [ 1.0, -1.0, -1.0], [ 1.0, -1.0,  1.0],
    [ 1.0,  1.0, -1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0,  1.0],

    // –X (left) face, inward normal (+1,0,0)
    [-1.0,  1.0,  1.0], [-1.0, -1.0,  1.0], [-1.0, -1.0, -1.0],
    [-1.0,  1.0,  1.0], [-1.0, -1.0, -1.0], [-1.0,  1.0, -1.0],

    // +Y (top) face, inward normal (0,‑1,0)
    [-1.0,  1.0, -1.0], [ 1.0,  1.0, -1.0], [ 1.0,  1.0,  1.0],
    [-1.0,  1.0, -1.0], [ 1.0,  1.0,  1.0], [-1.0,  1.0,  1.0],

    // –Y (bottom) face, inward normal (0,+1,0)
    [-1.0, -1.0, -1.0], [-1.0, -1.0,  1.0], [ 1.0, -1.0,  1.0],
    [-1.0, -1.0, -1.0], [ 1.0, -1.0,  1.0], [ 1.0, -1.0, -1.0],

    // +Z (front) face, inward normal (0,0,‑1)
    [-1.0,  1.0,  1.0], [ 1.0,  1.0,  1.0], [ 1.0, -1.0,  1.0],
    [-1.0,  1.0,  1.0], [ 1.0, -1.0,  1.0], [-1.0, -1.0,  1.0],

    // –Z (back) face, inward normal (0,0,+1)
    [-1.0,  1.0, -1.0], [ 1.0, -1.0, -1.0], [ 1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0], [-1.0, -1.0, -1.0], [ 1.0, -1.0, -1.0],
];


// Total vertices: 36 (12 triangles)
pub struct SkyPass {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl Renderable for SkyPass {
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
            label: Some("Skybox Vertex Buffer"),
            contents: bytemuck::cast_slice(SKYBOX_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[&camera.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &device.create_shader_module(wgpu::include_wgsl!("skybox.wgsl")),
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
                module: &device.create_shader_module(wgpu::include_wgsl!("skybox.wgsl")),
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
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // No depth written
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
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
            pipeline,
            vertex_buffer,
        }
    }

    fn render(&mut self, pass: &mut wgpu::RenderPass, camera: &super::Camera) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &camera.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..36, 0..1);
    }
}
