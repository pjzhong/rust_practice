use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer},
    device::{Device, DeviceExtensions, Features, Queue},
    instance::{Instance, InstanceExtensions, PhysicalDevice},
    sync::GpuFuture,
};

mod compute_operations;
mod image_creation;

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

    compute_operations::run(device, queue);
    //buffer_create_example(device)
    //copy_operation(device, queue);
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
