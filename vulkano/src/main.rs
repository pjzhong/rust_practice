use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
    descriptor::{descriptor_set::PersistentDescriptorSet, PipelineLayoutAbstract},
    device::{Device, DeviceExtensions, Features, Queue},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
    pipeline::ComputePipeline,
    sync::GpuFuture,
};

fn main() {
    let instance =
        Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create instance");
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");
    println!("selected device:{:?}", physical.name());

    for family in physical.queue_families() {
        println!(
            "Found a {:?} with {:?} queues(s)",
            family,
            family.queues_count()
        );
    }

    let queue_family = physical
        .queue_families()
        .find(|q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                ..DeviceExtensions::none()
            },
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    //buffer_create_example(device)
    //copy_operation(device, queue);

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

#[allow(dead_code)]
fn copy_operation(device: Arc<Device>, queue: Arc<Queue>) {
    let source_content = 0..64;
    let source =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, source_content)
            .expect("failed to create buffer");

    let des_content = (0..64).map(|_| 0);
    let dest =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, des_content)
            .expect("failed to create buffer");

    let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
    builder.copy_buffer(source.clone(), dest.clone()).unwrap();
    let command_buffer = builder.build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let src_content = source.read().unwrap();
    let dest_content = dest.read().unwrap();

    assert_eq!(&*src_content, &*dest_content);
}

#[allow(dead_code)]
fn buffer_create_example(device: Arc<Device>) {
    let data = (5, false);
    let data_buffer =
        CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, data)
            .expect("failed to create buffer");

    let iter = (0..128).map(|_| 5u8);
    let iter_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, iter).unwrap();

    let data_content = data_buffer.write().unwrap();
    println!("{:?}-{:?}", data_content.0, data_content.1);

    let mut iter_content = iter_buffer.write().unwrap();
    iter_content[8] = 10;
    println!("{:?}", iter_content.iter())
}
