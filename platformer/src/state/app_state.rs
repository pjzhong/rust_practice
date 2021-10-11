use anyhow::*;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::Options;

use super::gpu_state::GpuState;

pub struct AppState {
    options: Options,
    pub gpu: GpuState,
}

impl AppState {
    pub fn new(window: &Window, mut gpu: GpuState, options: Options) -> Result<Self> {
        Ok(Self { options, gpu })
    }

    pub fn game_pad_input(&mut self, event: gilrs::Event) {
        unimplemented!()
    }

    pub fn event(&mut self, window: &Window, event: &winit::event::Event<()>) {
    }

    pub fn update(&mut self, window: &Window, dt: std::time::Duration) {
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SwapChainError> {
        Ok(())
    }

    pub fn resize(&mut self, window: &Window, new_size: winit::dpi::PhysicalSize<u32>) {
    }

    pub fn input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        false
    }
}
