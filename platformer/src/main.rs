use futures::executor::block_on;
use gilrs::Gilrs;
use structopt::StructOpt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod state;
mod texture;
mod util;

#[derive(StructOpt, Debug)]
pub struct Options {
    /// Display a debug overlay
    #[structopt(short, long)]
    pub debug_overlay: bool,

    /// Starts gameplay at specified checkpoint
    #[structopt(short, long)]
    pub checkpoint: Option<u32>,

    /// Number of lives to give player
    #[structopt(short, long, default_value = "3")]
    pub lives: u32,

    /// Palette to use; options are "gameboy", "mist", "nostalgia", and "nymph"
    #[structopt(short, long, default_value = "gameboy")]
    pub palette: String,

    /// Disables music
    #[structopt(short, long)]
    pub no_music: bool,
}

fn main() {
    let opt = Options::from_args();
    let event_loop = EventLoop::new();
    let builder = WindowBuilder::new().with_title("Gargoyle's Quest");

    #[cfg(target_os = "windows")]
    let builder =
        winit::platform::windows::WindowBuilderExtWindows::with_drag_and_drop(builder, false);

    let window = builder.build(&event_loop).unwrap();

    let mut gilrs = Gilrs::new().unwrap();
    for (_, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let gpu = block_on(state::gpu_state::GpuState::new(&window));
    let mut app_state = state::app_state::AppState::new(&window, gpu, opt).unwrap();
    let mut last_render_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        while let Some(event) = gilrs.next_event() {
            app_state.game_pad_input(event);
        }

        app_state.event(&window, &event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !app_state.input(&window, &event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                app_state.update(&window, dt);
                match app_state.render(&window) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => {
                        app_state.resize(&window, app_state.gpu.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(_) => {}
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
