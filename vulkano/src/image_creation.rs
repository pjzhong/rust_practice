use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder},
    device::{Device, Queue},
    format::{ClearValue, Format},
    image::{Dimensions, StorageImage},
};

#[allow(dead_code)]
pub fn run(device: Arc<Device>, queue: Arc<Queue>) {
    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 1024,
            height: 1024,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    builder
        .clear_color_image(image.clone(), ClearValue::Float([0.0, 0.0, 1.0, 1.0]))
        .unwrap();
    let command_buffer = builder.build().unwrap();
}
