use rand::{thread_rng, Rng};
use ray_tracing_in_one_weekend::camera::Camera;
use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use ray_tracing_in_one_weekend::clamp;
use ray_tracing_in_one_weekend::hittable::{HitRecord, Hittable};
use ray_tracing_in_one_weekend::material::{Dielectric, Lambertian, Metal};
use ray_tracing_in_one_weekend::ray::Ray;
use ray_tracing_in_one_weekend::sphere::Sphere;
use ray_tracing_in_one_weekend::vec::Vec3;
use ray_tracing_in_one_weekend::Color;

fn ray_color(r: &Ray, world: &[Box<dyn Hittable>], depth: i32) -> Color {
    if depth <= 0 {
        return Color::f32(0.0, 0.0, 0.0);
    }

    if let Some(mut rec) = hit(world, r, 0.001, f32::INFINITY) {
        return if let Some(material) = rec.material.clone() {
            if let Some((attenuation, scattered)) = material.scatter(r, &mut rec) {
                attenuation * ray_color(&scattered, world, depth - 1)
            } else {
                Color::default()
            }
        } else {
            Color::default()
        };
    }

    let unit_direction = r.dir().normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color::f32(1.0, 1.0, 1.0) + t * Color::f32(0.5, 0.7, 1.0)
}

fn hit(world: &[Box<dyn Hittable>], r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
    let mut rec = HitRecord::default();
    let mut hit_anything = false;
    let mut closet_so_far = t_max;

    for hittable in world {
        if hittable.hit(r, t_min, closet_so_far, &mut rec) {
            closet_so_far = rec.t;
            hit_anything = true;
        }
    }

    if hit_anything {
        Some(rec)
    } else {
        None
    }
}

fn main() {
    // Image
    let aspect_ration = 16.0 / 9.0;
    let image_width = 1280;
    let image_height = (image_width as f32 / aspect_ration) as i32;
    let samples_per_pixel = 100;
    let max_depth = 50;

    let material_ground = Rc::new(Lambertian::new(Color::f32(0.8, 0.8, 0.0)));
    let material_center = Rc::new(Lambertian::new(Color::f32(0.1, 0.2, 0.5)));
    // let material_left = Rc::new(Metal::new(Color::f32(0.8, 0.8, 0.8), 0.1));
    let material_left = Rc::new(Dielectric::new(1.5));
    let material_right = Rc::new(Metal::new(Color::f32(0.8, 0.6, 0.2), 0.1));

    // World
    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::new(
            Vec3::f32(0.0, -100.5, -1.0),
            100.0,
            material_ground.clone(),
        )),
        Box::new(Sphere::new(
            Vec3::f32(0.0, 0.0, -1.0),
            0.5,
            material_center.clone(),
        )),
        Box::new(Sphere::new(
            Vec3::f32(-1.0, 0.0, -1.0),
            0.5,
            material_left.clone(),
        )),
        Box::new(Sphere::new(
            Vec3::f32(-1.0, 0.0, -1.0),
            -0.4,
            material_left.clone(),
        )),
        Box::new(Sphere::new(
            Vec3::f32(1.0, 0.0, -1.0),
            0.5,
            material_right.clone(),
        )),
    ];

    // Camera
    let camera = Camera::new(
        Vec3::f32(-2.0, 2.0, 1.0),
        Vec3::f32(0.0, 0.0, -1.0),
        Vec3::f32(0.0, 1.0, 0.0),
        90.0,
        aspect_ration,
    );

    // Render
    let current_dir = env::current_dir().unwrap();
    let mut file = File::create(current_dir.join("blue_to_white.ppm")).unwrap();

    file.write_all(format!("P3\n{} {}\n255\n", image_width, image_height).as_bytes())
        .unwrap();

    let mut range = thread_rng();
    for j in (0..=image_height - 1).rev() {
        println!("Scan lines remaining:{}", j);
        for i in 0..image_width {
            let mut pixel_color = Vec3::f32(0.0, 0.0, 0.0);
            for _ in 0..samples_per_pixel {
                let u = (i as f32 + range.gen_range(0.0..1.0)) / (image_width - 1) as f32;
                let v = (j as f32 + range.gen_range(0.0..1.0)) / (image_height - 1) as f32;
                let r = camera.get_ray(u, v);
                pixel_color += ray_color(&r, &world, max_depth);
            }

            write_color(&mut file, &pixel_color, samples_per_pixel)
        }
    }
}

fn write_color(f: &mut File, pixel_color: &Vec3<f32>, samples_per_pixel: i32) {
    let scale = 1.0 / samples_per_pixel as f32;
    let r = (pixel_color.x * scale).sqrt();
    let g = (pixel_color.y * scale).sqrt();
    let b = (pixel_color.z * scale).sqrt();

    f.write_all(
        format!(
            "{} {} {}\n",
            (256.0 * clamp(r, 0.0, 0.999)) as i32,
            (256.0 * clamp(g, 0.0, 0.999)) as i32,
            (256.0 * clamp(b, 0.0, 0.999)) as i32,
        )
        .as_bytes(),
    )
    .unwrap();
}
