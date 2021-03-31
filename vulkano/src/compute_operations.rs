use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
    descriptor::{descriptor_set::PersistentDescriptorSet, PipelineLayoutAbstract},
    device::{Device, Queue},
    pipeline::ComputePipeline,
    sync::GpuFuture,
};

#[allow(dead_code)]
pub fn run(device: Arc<Device>, queue: Arc<Queue>) {
    let data_iter = 0..65536;
    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, data_iter)
            .expect("failed to create buffer");

    let shader = cs::Shader::load(device.clone()).expect("failed to create shader module");
    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &(), None)
            .expect("failed to create pipeline"),
    );

    let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
    let set = Arc::new(
        PersistentDescriptorSet::start(layout.clone())
            .add_buffer(data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    builder
        .dispatch(
            [1024, 1, 1],
            compute_pipeline.clone(),
            set.clone(),
            (),
            0..0,
        )
        .unwrap();
    let command_buffer = builder.build().unwrap();
    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12)
    }
}

mod cs {
    vulkano_shaders::shader! {
    ty:"compute",
    src: "
 #version 450

 layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

 layout(set = 0, binding = 0) buffer Data {
     uint data[];
 } buf;

 void main() {
     uint idx = gl_GlobalInvocationID.x;
     buf.data[idx] *= 12;
 }
 "
    }
}
