use crate::context::ContextState;
use winit::event::WindowEvent;

use super::Camera;

pub trait Renderable {
    fn render(
        &mut self,
        pass: &mut wgpu::RenderPass,
        camera: &Camera
    );

    fn input(&mut self, event: &WindowEvent, context: &ContextState) -> bool;

    fn update(&mut self, context: &mut ContextState, queue: &wgpu::Queue);

    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, camera: &Camera) -> Self
    where
        Self: Sized;
}
