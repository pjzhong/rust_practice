use cgmath::{point2, vec2, Point2, Vector2};

use crate::texture::Texture;
use crate::state::gpu_state;
use crate::Options;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LcdUniformData {
    camera_position: Point2<f32>,
    viewport_size: Vector2<f32>,
    pixels_per_unit: Vector2<f32>,
    pixel_effect_alpha: f32,
    shadow_effect_alpha: f32,
}

unsafe impl bytemuck::Pod for LcdUniformData {}
unsafe impl bytemuck::Zeroable for LcdUniformData {}

impl Default for LcdUniformData {
    fn default() -> Self {
        Self {
            camera_position: point2(0.0, 0.0),
            viewport_size: vec2(1.0, 1.0),
            pixels_per_unit: vec2(1.0, 1.0),
            pixel_effect_alpha: 1.0,
            shadow_effect_alpha: 1.0,
        }
    }
}

impl LcdUniformData {
    pub fn set_pixel_effect_alpha(&mut self, pixel_effect_alpha: f32) -> &mut Self {
        self.pixel_effect_alpha = pixel_effect_alpha;
        self
    }

    pub fn set_shadow_effect_alpha(&mut self, shadow_effect_alpha: f32) -> &mut Self {
        self.shadow_effect_alpha = shadow_effect_alpha;
        self
    }

    pub fn set_camera_position(&mut self, camera_position: Point2<f32>) -> &mut Self {
        self.camera_position = camera_position;
        self
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector2<f32>) -> &mut Self {
        self.viewport_size = viewport_size;
        self
    }

    pub fn set_pixels_per_unit(&mut self, pixels_per_unit: Vector2<f32>) -> &mut Self {
        self.pixels_per_unit = pixels_per_unit;
        self
    }
}

pub type LcdUniforms = crate::util::UniformWrapper<LcdUniformData>;

pub struct LcdFilter {
    textures_bind_group_layout: wgpu::BindGroupLayout,
    textures_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    tonemap: Texture,
    uniforms: LcdUniforms,
}

impl LcdFilter {
}
