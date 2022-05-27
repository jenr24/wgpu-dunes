mod state;
mod vertex;
mod image;

mod compute;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::Element;

use crate::state::*;

fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }
}

#[wasm_bindgen]
pub struct Canvas {
    event_loop: EventLoop<()>,
    window: Window,
}

// *********************
// *** API functions ***
// *********************

#[wasm_bindgen]
pub fn create_canvas() -> Canvas {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")] {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));
    }
    
    Canvas { event_loop, window }
}

#[wasm_bindgen]
pub fn get_canvas_element(canvas: &Canvas) -> Element {
    use winit::platform::web::WindowExtWebSys;
    web_sys::Element::from(canvas.window.canvas())
}

#[wasm_bindgen]
pub async fn run_canvas_loop(app: Canvas) {
    let window = app.window;
    let event_loop = app.event_loop;
    run_window_event_loop(window, event_loop).await
}

// *********************
// *********************
// *********************

#[cfg(target_arch = "wasm32")]
fn append_to_document(element: &Element) {
    web_sys::window()
        .and_then(|win: web_sys::Window| win.document())
        .and_then(|doc: web_sys::Document| {
            let dst = doc.get_element_by_id("root")?;
            dst.append_child(element).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

pub async fn run_window_event_loop(window: Window, event_loop: EventLoop<()>) {
    let mut state = State::new(&window).await.unwrap();
    event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !state.input(event) {
                        // UPDATED!
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &&mut so w have to dereference it twice
                                state.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        
                        Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                    }
                }
                Event::RedrawEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        });   
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    init_logger();

    let canvas = create_canvas();

    // TODO: export canvas element 
    #[cfg(target_arch = "wasm32")] {
        let canvas = get_canvas_element(&canvas);
        append_to_document(&canvas);
    }

    run_canvas_loop(canvas).await;
}