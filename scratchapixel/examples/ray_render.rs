use scratchapixel::{ray_tracer, Sphere};
use scratchapixel::vec::Vec3;

/// thanks [`scratchapixel`]
/// This code is copy from ['raytracer'] and translate to rust
///
/// [`scratchapixel`]: https://www.scratchapixel.com/index.php/
/// [`raytracer`]: https://www.scratchapixel.com/code.php?id=3&origin=/lessons/3d-basic-rendering/introduction-to-ray-tracing
fn main() {
    let spheres = vec![
        Sphere::new(
            Vec3::f32(0.0, -10004.0, -20.0),
            Vec3::f32(0.20, 0.20, 0.20),
            Vec3::<f32>::default(),
            10000.0,
            0.0,
            0.0,
        ),
        Sphere::new(
            Vec3::f32(0.0, 0.0, -20.0),
            Vec3::f32(1.00, 0.32, 0.36),
            Vec3::<f32>::default(),
            4.0,
            1.0,
            0.5,
        ),
        Sphere::new(
            Vec3::f32(5.0, -1.0, -15.0),
            Vec3::f32(0.90, 0.76, 0.46),
            Vec3::<f32>::default(),
            2.0,
            1.0,
            0.0,
        ),
        Sphere::new(
            Vec3::f32(5.0, 0.0, -25.0),
            Vec3::f32(0.65, 0.77, 0.97),
            Vec3::<f32>::default(),
            3.0,
            1.0,
            0.0,
        ),
        Sphere::new(
            Vec3::f32(-5.5, 0.0, -15.0),
            Vec3::f32(0.90, 0.90, 0.90),
            Vec3::<f32>::default(),
            3.0,
            1.0,
            0.0,
        ),
        Sphere::new(
            Vec3::f32(0.0, 20.0, -30.0),
            Vec3::f32(0.00, 0.00, 0.00),
            Vec3::f32(3.0, 3.0, 3.0),
            3.0,
            0.0,
            0.0,
        ),
    ];

    ray_tracer::render(&spheres);
}
