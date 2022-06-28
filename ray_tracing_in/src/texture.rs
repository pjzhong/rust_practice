use crate::perlin::Perlin;
use crate::{clamp, Color, Point};
use stb_image::image::LoadResult::{Error, ImageF32, ImageU8};
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

pub trait Texture: Sync + Send {
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
    odd: Arc<dyn Texture>,
    even: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn with_texture(odd: Arc<dyn Texture>, even: Arc<dyn Texture>) -> Self {
        Self { odd, even }
    }

    pub fn with_color(odd: Color, even: Color) -> Self {
        Self {
            odd: Arc::new(SolidColor::new(odd)),
            even: Arc::new(SolidColor::new(even)),
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

pub struct ImageTexture {
    data: Vec<u8>,
    width: usize,
    height: usize,
    depth: usize,
}

impl ImageTexture {
    pub fn new<T: AsRef<Path> + Debug>(file_name: T) -> Self {
        let (data, width, height, depth) = match stb_image::image::load(&file_name) {
            ImageU8(image) => (image.data, image.width, image.height, image.depth),
            ImageF32(_) => {
                println!("load {:?} as f32, not support now", file_name);
                (vec![], 0, 0, 0)
            }
            Error(str) => {
                println!("load {:?} failed, reason:{:?}", file_name, str);
                (vec![], 0, 0, 0)
            }
        };

        Self {
            data,
            width,
            height,
            depth,
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f32, v: f32, _: &Point) -> Color {
        if self.data.is_empty() {
            return Color::f32(0.0, 1.0, 1.0);
        }

        let u = clamp(u, 0.0, 1.0);
        let v = 1.0 - clamp(v, 0.0, 1.0);

        let i = (u * self.width as f32) as usize;
        let j = (v * self.height as f32) as usize;

        let i = if i >= self.width { self.width - 1 } else { i };
        let j = if j >= self.height { self.height - 1 } else { j };

        let color_scale = 1.0 / 255.0;

        //why this work
        //copy from:
        // https://github.com/fralken/ray-tracing-the-next-week/blob/80c9675c4ee3ccb851f407c386c2cad62122a124/src/texture.rs#L78
        let idx = i * self.depth + j * self.depth * self.width;

        Color::f32(
            color_scale * self.data[idx] as f32,
            color_scale * self.data[idx + 1] as f32,
            color_scale * self.data[idx + 2] as f32,
        )
    }
}
