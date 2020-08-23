#![allow(dead_code)]

use winit::{
    event,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod gfx;
mod state;
mod util;

use gfx::prelude::*;

async fn run(event_loop: EventLoop<()>, window: Window) {
    env_logger::init();

    let mut world = state::world::World::new();

    // Initialize the gfx context.
    let mut gfx_context = GfxContext::create(&window).await.unwrap();
    let mut world_renderer = WorldRenderer::new(world.id);

    // Start focused by default, assuming the application was executed with the intention of using
    // it straight away.
    let mut window_focused: bool = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) =>
                gfx_context.render(&mut world, &mut world_renderer),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } =>
                gfx_context.resize(size),
            // Handle requests to close the window...
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } |
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input: event::KeyboardInput {
                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                state: event::ElementState::Pressed, ..
            }, .. }, .. } => {
                *control_flow = ControlFlow::Exit;

                window.set_cursor_grab(false).unwrap();
                window.set_cursor_visible(true);
            },

            // We track if the window has focus so that we can ignore device events when focus is
            // lost.
            Event::WindowEvent { event: WindowEvent::Focused(b), .. } => window_focused = b,

            Event::WindowEvent { event: WindowEvent::CursorEntered { .. }, .. } => {
                window.set_cursor_grab(true).unwrap();
                window.set_cursor_visible(false);
            },
            Event::WindowEvent { event: WindowEvent::CursorLeft { .. }, .. } => {
                window.set_cursor_grab(false).unwrap();
                window.set_cursor_visible(true);
            },

            // Ignore all device events if the window does not have focus.
            Event::DeviceEvent { .. } if !window_focused => {}
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_inner_size(winit::dpi::PhysicalSize::new(1280, 720));
    futures::executor::block_on(run(event_loop, window));
}
