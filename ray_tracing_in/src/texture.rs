use crate::perlin::Perlin;
use crate::{Color, Point};
use std::rc::Rc;

pub trait Texture {
    fn value(&self, u: f32, v: f32, p: &Point) -> Color;
}

pub struct SolidColor {
    color: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Texture for SolidColor {
    fn value(&self, _: f32, _: f32, _: &Point) -> Color {
        self.color
    }
}

pub struct CheckerTexture {
    odd: Rc<dyn Texture>,
    even: Rc<dyn Texture>,
}

impl CheckerTexture {
    pub fn with_texture(odd: Rc<dyn Texture>, even: Rc<dyn Texture>) -> Self {
        Self { odd, even }
    }

    pub fn with_color(odd: Color, even: Color) -> Self {
        Self {
            odd: Rc::new(SolidColor::new(odd)),
            even: Rc::new(SolidColor::new(even)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f32, v: f32, p: &Point) -> Color {
        let sines = (10.0 * p.x).sin() * (10.0 * p.y).sin() * (10.0 * p.z).sin();
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

pub struct NoiseTexture {
    noise: Perlin,
    scale: f32,
}

impl Default for NoiseTexture {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl NoiseTexture {
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            noise: Perlin::default(),
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _: f32, _: f32, p: &Point) -> Color {
        Color::f32(1.0, 1.0, 1.0)
            * 0.5
            * (1.0 + (self.scale * p.z + 10.0 * self.noise.turb(p, 7)).sin())
    }
}
