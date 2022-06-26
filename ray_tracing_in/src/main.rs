use rand::{thread_rng, Rng};
use ray_tracing_in::camera::Camera;
use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use ray_tracing_in::clamp;
use ray_tracing_in::hittable::Hittable;
use ray_tracing_in::material::{Dielectric, Lambertian, Metal};
use ray_tracing_in::ray::Ray;
use ray_tracing_in::sphere::Sphere;
use ray_tracing_in::texture::{CheckerTexture, NoiseTexture};
use ray_tracing_in::vec::Vec3;
use ray_tracing_in::Color;

fn ray_color(r: &Ray, world: &[Rc<dyn Hittable>], depth: i32) -> Color {
    if depth <= 0 {
        return Color::f32(1.0, 1.0, 1.0);
    }

    if let Some(mut rec) = world.hit(r, 0.001, f32::INFINITY) {
        return if let Some(material) = rec.material.clone() {
            if let Some((color, scattered)) = material.scatter(r, &mut rec) {
                color * ray_color(&scattered, world, depth - 1)
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

fn random_scene() -> Vec<Rc<dyn Hittable>> {
    let mut world: Vec<Rc<dyn Hittable>> = Vec::new();
    world.push(Rc::new(Sphere::steady(
        Vec3::f32(0.0, -1000., 0.0),
        1000.0,
        Rc::new(Lambertian::with_texture(Rc::new(
            CheckerTexture::with_color(Color::f32(0.2, 0.3, 0.1), Color::f32(0.9, 0.9, 0.9)),
        ))),
    )));

    let mut rang = thread_rng();
    let point = Vec3::f32(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rang.gen_range(0.0..1.0f32);

            let center = Vec3::f32(
                a as f32 + 0.9 * rang.gen_range(0.0..1.0),
                0.2,
                b as f32 + 0.9 * rang.gen_range(0.0..1.0),
            );

            if (center - point).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::random() * Color::random();
                    let center2 = center + Vec3::f32(0.0, rang.gen_range(0.0..0.5), 0.0);
                    world.push(Rc::new(Sphere::motion(
                        center,
                        center2,
                        0.2,
                        0.0,
                        1.0,
                        Rc::new(Lambertian::with_color(albedo)),
                    )))
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = rang.gen_range(0.0..0.5);
                    world.push(Rc::new(Sphere::steady(
                        center,
                        0.2,
                        Rc::new(Metal::new(albedo, fuzz)),
                    )))
                } else {
                    world.push(Rc::new(Sphere::steady(
                        center,
                        0.2,
                        Rc::new(Dielectric::new(1.5)),
                    )))
                }
            }
        }
    }

    world.push(Rc::new(Sphere::steady(
        Vec3::f32(0.0, 1.0, 0.0),
        1.0,
        Rc::new(Dielectric::new(1.5)),
    )));
    world.push(Rc::new(Sphere::steady(
        Vec3::f32(-4.0, 1.0, 0.0),
        1.0,
        Rc::new(Lambertian::with_color(Color::f32(0.4, 0.2, 0.1))),
    )));
    world.push(Rc::new(Sphere::steady(
        Vec3::f32(4.0, 1.0, 0.0),
        1.0,
        Rc::new(Metal::new(Color::f32(0.7, 0.6, 0.5), 0.0)),
    )));

    world
}

fn two_spheres() -> Vec<Rc<dyn Hittable>> {
    let material_ground = Rc::new(Lambertian::with_texture(Rc::new(
        CheckerTexture::with_color(Color::f32(0.2, 0.3, 0.1), Color::f32(0.9, 0.9, 0.9)),
    )));
    let world: Vec<Rc<dyn Hittable>> = vec![
        Rc::new(Sphere::steady(
            Vec3::f32(0.0, -10., 0.0),
            10.0,
            material_ground.clone(),
        )),
        Rc::new(Sphere::steady(
            Vec3::f32(0.0, 10., 0.0),
            10.0,
            material_ground,
        )),
    ];

    world
}

fn two_perline_spheres() -> Vec<Rc<dyn Hittable>> {
    let material_ground = Rc::new(Lambertian::with_texture(Rc::new(NoiseTexture::default())));
    let world: Vec<Rc<dyn Hittable>> = vec![
        Rc::new(Sphere::steady(
            Vec3::f32(0.0, -1000., 0.0),
            1000.0,
            material_ground.clone(),
        )),
       Rc::new(Sphere::steady(
            Vec3::f32(0.0, 2., 0.0),
            2.0,
            material_ground,
        )),
    ];

    world
}

fn main() {
    // Image
    let aspect_ration = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f32 / aspect_ration) as i32;
    let samples_per_pixel = 100;
    let max_depth = 50;

    // World And Camera
    let case = 2;
    let (world, look_from, look_at, aperture, vfov) = match case {
        1 => (
            two_spheres(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.0,
            20.0,
        ),
        2 => (
            two_perline_spheres(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
        ),
        _ => (
            random_scene(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
        ),
    };

    let vup = Vec3::f32(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;

    let camera = Camera::new(
        look_from,
        look_at,
        vup,
        vfov,
        aspect_ration,
        aperture,
        dist_to_focus,
        0.0,
        1.0,
    );

    // Render
    let current_dir = env::current_dir().unwrap();
    let mut file = File::create(current_dir.join(format!("{}.ppm", case))).unwrap();

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
