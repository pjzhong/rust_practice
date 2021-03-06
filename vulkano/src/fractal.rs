use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
    device::{Device, Queue},
    format::{Format},
    image::{Dimensions, StorageImage},
    pipeline::ComputePipeline,
    sync::GpuFuture,
};

#[allow(dead_code)]
pub fn run(device: Arc<Device>, queue: Arc<Queue>) {
    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 2048,
            height: 2048,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None)
            .expect("failed to create compute pipeline"),
    );

    let set = Arc::new(
        PersistentDescriptorSet::start(
            compute_pipeline
                .layout()
                .descriptor_set_layout(0)
                .unwrap()
                .clone(),
        )
        .add_image(image.clone())
        .unwrap()
        .build()
        .unwrap(),
    );

    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..2048 * 2048 * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    builder
        .dispatch(
            [2048 / 8, 2048 / 8, 1],
            compute_pipeline.clone(),
            set.clone(),
            (),
            0..0,
        )
        .unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(2048, 2048, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
}

mod cs {
    vulkano_shaders::shader! {
    ty:"compute",
    src: "
 #version 450

 layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

 layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

 void main() {
     vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
     vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

     vec2 z = vec2(0.0, 0.0);
     float i;
     for (i = 0.0; i < 1.0; i += 0.005) {
         z = vec2(
             z.x * z.x - z.y * z.y + c.x,
             z.y * z.x + z.x * z.y + c.y
         );

         if (length(z) > 4.0) {
             break;
         }
     }

     vec4 to_write = vec4(vec3(i), 1.0);
     imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
 }
 "
    }
}
