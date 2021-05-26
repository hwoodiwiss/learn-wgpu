use futures::executor::block_on;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod state;
mod vertex;
mod texture;
mod camera;
mod uniform;

use state::State;

fn main() {
    env_logger::init();

    let evt_loop = EventLoop::new();

    let window_size = PhysicalSize::new(1920, 1080);
    let window = WindowBuilder::new()
        .with_title("WGPU Rendering")
        .with_inner_size(window_size)
        .build(&evt_loop)
        .expect("Failed to create window!");

    let mut render_state = block_on(State::new(&window));
    evt_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() && !render_state.input(event) => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(new_size) => render_state.resize(*new_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                render_state.resize(**new_inner_size)
            }
            WindowEvent::KeyboardInput { input, .. } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Escape),
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        },
        Event::RedrawRequested(_) => {
            render_state.update();
            match render_state.render() {
                Ok(_) => {}
                //On swapchain lost, recreate
                Err(wgpu::SwapChainError::Lost) => render_state.resize(render_state.size),
                // On OOM Exit
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
