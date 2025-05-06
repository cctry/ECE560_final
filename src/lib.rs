use wasm_bindgen::prelude::*;
use winit::{
    error::EventLoopError,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod context;
use context::Context;
mod render;

fn create_window(title: &str) -> Result<(Window, EventLoop<()>), EventLoopError> {
    let event_loop = EventLoop::new()?;
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowBuilderExtWebSys;
    let canvas = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id("canvas"))
        .and_then(|elem| elem.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .expect("Failed to get canvas element");
    let window = WindowBuilder::new()
        .with_canvas(Some(canvas))
        .with_title(title)
        .build(&event_loop)?;
    Ok((window, event_loop))
}

#[allow(unused)]
fn register_passes(context: &mut Context) {
    use render::PerlinPass;
    use render::SkyPass;
    context.add_render_pass::<SkyPass>();
    context.add_render_pass::<PerlinPass>();
}

#[wasm_bindgen(start)]
async fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    let (window, event_loop) = create_window("Hello Winit").expect("Failed to create window");
    let mut context = Context::new(&window).await;
    register_passes(&mut context);

    let mut surface_configured = false;
    let mut last_render_time = instant::Instant::now();

    let _ = event_loop
        .run(move |event, control_flow| {
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if context.is_cursor_captured() {
                        context
                            .camera
                            .process_mouse((delta.0 as f32, delta.1 as f32).into());
                    }
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == context.window().id() => {
                    if !context.input(event) {
                        // If the event was not handled, we can pass it to the window
                        match event {
                            WindowEvent::CloseRequested => {
                                log::info!("Close requested");
                                control_flow.exit();
                            }
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => {
                                if context.is_cursor_captured() {
                                    context.toggle_cursor_capture();
                                }
                            }
                            WindowEvent::MouseInput {
                                state: ElementState::Pressed,
                                button: MouseButton::Left,
                                ..
                            } => {
                                context.toggle_cursor_capture(); // click to switch
                            }
                            WindowEvent::Resized(physical_size) => {
                                log::info!("physical_size: {physical_size:?}");
                                surface_configured = true;
                                context.resize(Some(*physical_size));
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                context.window().request_redraw();

                                if !surface_configured {
                                    return;
                                }

                                let now = instant::Instant::now();
                                let dt = now - last_render_time;
                                last_render_time = now;

                                log::info!("FPS: {:.2}", 1.0 / dt.as_secs_f32());

                                context.update(&dt);
                                match context.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => context.resize(None),
                                    // The system is out of memory, we should probably quit
                                    Err(
                                        wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other,
                                    ) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
