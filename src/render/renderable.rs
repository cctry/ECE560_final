use crate::context::ContextData;
use winit::event::WindowEvent;

pub trait Renderable {
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

    fn input(&mut self, event: &WindowEvent, context: &ContextData)-> bool;

    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    where
        Self: Sized;
}
