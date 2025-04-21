// pub struct TrianglePass {
//     pipeline: wgpu::RenderPipeline,
//     vertex_buffer: wgpu::Buffer,
//     num_vertices: u32,
//     // Add bind groups etc. as needed
// }

// impl TrianglePass {
//     pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
//         // Load shader
//         let shader = device.create_shader_module(wgpu::include_wgsl!("triangle.wgsl"));

//         // Create pipeline
//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("Triangle Pipeline Layout"),
//             bind_group_layouts: &[],
//             push_constant_ranges: &[],
//         });

//         let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("Triangle Pipeline"),
//             layout: Some(&pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: &shader,
//                 entry_point: "vs_main",
//                 buffers: &[/* define vertex buffer layout */],
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: &shader,
//                 entry_point: "fs_main",
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: config.format,
//                     blend: Some(wgpu::BlendState::REPLACE),
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//             }),
//             primitive: wgpu::PrimitiveState::default(),
//             depth_stencil: None,
//             multisample: wgpu::MultisampleState::default(),
//             multiview: None,
//         });

//         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Triangle Vertex Buffer"),
//             contents: bytemuck::cast_slice(&[
//                 // your vertex data here
//             ]),
//             usage: wgpu::BufferUsages::VERTEX,
//         });

//         Self {
//             pipeline,
//             vertex_buffer,
//             num_vertices: 3,
//         }
//     }
// }

// impl Renderable for TrianglePass {
//     fn render(
//         &mut self,
//         encoder: &mut wgpu::CommandEncoder,
//         view: &wgpu::TextureView,
//         _device: &wgpu::Device,
//         _queue: &wgpu::Queue,
//     ) {
//         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//             label: Some("Triangle Render Pass"),
//             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                 view,
//                 resolve_target: None,
//                 ops: wgpu::Operations {
//                     load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
//                     store: true,
//                 },
//             })],
//             depth_stencil_attachment: None,
//         });

//         render_pass.set_pipeline(&self.pipeline);
//         render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
//         render_pass.draw(0..self.num_vertices, 0..1);
//     }
// }

use super::renderable::Renderable;
use crate::context::ContextData;
use winit::event::WindowEvent;

pub struct ClearPass {
    clear_color: wgpu::Color,
}

impl Renderable for ClearPass {
    fn new(_device: &wgpu::Device, _config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    fn input(&mut self, event: &WindowEvent, context: &ContextData) -> bool {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.clear_color = wgpu::Color {
                r: position.x as f64 / context.size().width as f64,
                g: position.y as f64 / context.size().height as f64,
                b: 1.0,
                a: 1.0,
            };
            return true;
        }
        false
    }
}
