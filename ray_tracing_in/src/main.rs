use rand::{thread_rng, Rng};
use ray_tracing_in::camera::Camera;
use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use ray_tracing_in::hittable::Hittable;
use ray_tracing_in::material::{Dielectric, DiffuseLight, Lambertian, Metal};
use ray_tracing_in::ray::Ray;
use ray_tracing_in::rectangle::XyRectangle;
use ray_tracing_in::sphere::Sphere;
use ray_tracing_in::texture::{CheckerTexture, ImageTexture, NoiseTexture};
use ray_tracing_in::vec::Vec3;
use ray_tracing_in::Color;
use ray_tracing_in::{clamp, Point};

fn ray_color(r: &Ray, background: &Color, world: &[Arc<dyn Hittable>], depth: i32) -> Color {
    if depth <= 0 {
        return *background;
    }

    if let Some(rec) = world.hit(r, 0.001, f32::INFINITY) {
        let material = rec.material.clone();
        let emitted = material.emitted(rec.u, rec.v, &rec.p);
        if let Some((color, scattered)) = material.scatter(r, &rec) {
            emitted + color * ray_color(&scattered, background, world, depth - 1)
        } else {
            emitted
        }
    } else {
        *background
    }
}

fn random_scene() -> Vec<Arc<dyn Hittable>> {
    let mut world: Vec<Arc<dyn Hittable>> = vec![Arc::new(Sphere::steady(
        Vec3::f32(0.0, -1000., 0.0),
        1000.0,
        Arc::new(Lambertian::with_texture(Box::new(
            CheckerTexture::with_color(Color::f32(0.2, 0.3, 0.1), Color::f32(0.9, 0.9, 0.9)),
        ))),
    ))];

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
                    world.push(Arc::new(Sphere::motion(
                        center,
                        center2,
                        0.2,
                        0.0,
                        1.0,
                        Arc::new(Lambertian::with_color(albedo)),
                    )))
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = rang.gen_range(0.0..0.5);
                    world.push(Arc::new(Sphere::steady(
                        center,
                        0.2,
                        Arc::new(Metal::new(albedo, fuzz)),
                    )))
                } else {
                    world.push(Arc::new(Sphere::steady(
                        center,
                        0.2,
                        Arc::new(Dielectric::new(1.5)),
                    )))
                }
            }
        }
    }

    world.push(Arc::new(Sphere::steady(
        Vec3::f32(0.0, 1.0, 0.0),
        1.0,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.push(Arc::new(Sphere::steady(
        Vec3::f32(-4.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::with_color(Color::f32(0.4, 0.2, 0.1))),
    )));
    world.push(Arc::new(Sphere::steady(
        Vec3::f32(4.0, 1.0, 0.0),
        1.0,
        Arc::new(Metal::new(Color::f32(0.7, 0.6, 0.5), 0.0)),
    )));

    world
}

fn two_spheres() -> Vec<Arc<dyn Hittable>> {
    let material_ground = Arc::new(Lambertian::with_texture(Box::new(
        CheckerTexture::with_color(Color::f32(0.2, 0.3, 0.1), Color::f32(0.9, 0.9, 0.9)),
    )));
    let world: Vec<Arc<dyn Hittable>> = vec![
        Arc::new(Sphere::steady(
            Vec3::f32(0.0, -10., 0.0),
            10.0,
            material_ground.clone(),
        )),
        Arc::new(Sphere::steady(
            Vec3::f32(0.0, 10., 0.0),
            10.0,
            material_ground,
        )),
    ];

    world
}

fn two_perline_spheres() -> Vec<Arc<dyn Hittable>> {
    let material_ground = Arc::new(Lambertian::with_texture(Box::new(NoiseTexture::new(4.0))));
    let world: Vec<Arc<dyn Hittable>> = vec![
        Arc::new(Sphere::steady(
            Vec3::f32(0.0, -1000., 0.0),
            1000.0,
            material_ground.clone(),
        )),
        Arc::new(Sphere::steady(
            Vec3::f32(0.0, 2., 0.0),
            2.0,
            material_ground,
        )),
    ];

    world
}

fn earth() -> Vec<Arc<dyn Hittable>> {
    let path = env::current_dir().unwrap().join("earthmap.jpg");
    let earth_texture = Box::new(ImageTexture::new(path));
    let earth_surface = Arc::new(Lambertian::with_texture(earth_texture));
    let globe = Arc::new(Sphere::steady(
        Point::f32(0.0, 0.0, 0.0),
        2.0,
        earth_surface,
    ));

    vec![globe]
}

fn simple_light() -> Vec<Arc<dyn Hittable>> {
    let material_ground = Arc::new(Lambertian::with_texture(Box::new(NoiseTexture::new(4.0))));
    let world: Vec<Arc<dyn Hittable>> = vec![
        Arc::new(Sphere::steady(
            Vec3::f32(0.0, -1000., 0.0),
            1000.0,
            material_ground.clone(),
        )),
        Arc::new(Sphere::steady(
            Vec3::f32(0.0, 2., 0.0),
            2.0,
            material_ground,
        )),
        Arc::new(XyRectangle::new(
            3.0,
            5.0,
            1.0,
            3.0,
            -2.0,
            Arc::new(DiffuseLight::new(Color::f32(4., 4., 4.))),
        )),
    ];

    world
}

fn main() {
    // Image
    let aspect_ration = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f32 / aspect_ration) as i32;
    let samples_per_pixel = 500;
    let max_depth = 50;

    // World And Camera
    let case = 5;
    let (world, look_from, look_at, aperture, vfov, background) = match case {
        1 => (
            two_spheres(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
        ),
        2 => (
            two_perline_spheres(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
        ),
        3 => (
            earth(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
        ),
        4 => (
            random_scene(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
        ),
        5 => (
            simple_light(),
            Vec3::f32(26.0, 3.0, 6.0),
            Vec3::f32(0.0, 2.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.0, 0.0, 0.00),
        ),
        _ => (
            vec![],
            Vec3::default(),
            Vec3::default(),
            0.0,
            0.0,
            Color::default(),
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
                pixel_color += ray_color(&r, &background, &world, max_depth);
            }

            //let col: Vec3<f32> = (0..samples_per_pixel).map(|_| )

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
