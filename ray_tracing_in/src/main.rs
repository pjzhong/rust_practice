use rand::{thread_rng, Rng};
use ray_tracing_in::camera::Camera;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use ray_tracing_in::hittable::{Hittable, RotateY, Translate};
use ray_tracing_in::material::{Dielectric, DiffuseLight, Lambertian, Metal};
use ray_tracing_in::ray::Ray;
use ray_tracing_in::rectangle::{XyRectangle, XzRectangle, YzRectangle};
use ray_tracing_in::sphere::Sphere;
use ray_tracing_in::texture::{CheckerTexture, ImageTexture, NoiseTexture};
use ray_tracing_in::vec::Vec3;
use ray_tracing_in::{clamp, Boxes, Point};
use ray_tracing_in::{Color, Image};

fn ray_color(r: &Ray, background: &Color, world: &[Box<dyn Hittable>], depth: i32) -> Color {
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

fn random_scene() -> Vec<Box<dyn Hittable>> {
    let mut world: Vec<Box<dyn Hittable>> = vec![Box::new(Sphere::steady(
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
                    world.push(Box::new(Sphere::motion(
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
                    world.push(Box::new(Sphere::steady(
                        center,
                        0.2,
                        Arc::new(Metal::new(albedo, fuzz)),
                    )))
                } else {
                    world.push(Box::new(Sphere::steady(
                        center,
                        0.2,
                        Arc::new(Dielectric::new(1.5)),
                    )))
                }
            }
        }
    }

    world.push(Box::new(Sphere::steady(
        Vec3::f32(0.0, 1.0, 0.0),
        1.0,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.push(Box::new(Sphere::steady(
        Vec3::f32(-4.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::with_color(Color::f32(0.4, 0.2, 0.1))),
    )));
    world.push(Box::new(Sphere::steady(
        Vec3::f32(4.0, 1.0, 0.0),
        1.0,
        Arc::new(Metal::new(Color::f32(0.7, 0.6, 0.5), 0.0)),
    )));

    world
}

fn two_spheres() -> Vec<Box<dyn Hittable>> {
    let material_ground = Arc::new(Lambertian::with_texture(Box::new(
        CheckerTexture::with_color(Color::f32(0.2, 0.3, 0.1), Color::f32(0.9, 0.9, 0.9)),
    )));
    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::steady(
            Vec3::f32(0.0, -10., 0.0),
            10.0,
            material_ground.clone(),
        )),
        Box::new(Sphere::steady(
            Vec3::f32(0.0, 10., 0.0),
            10.0,
            material_ground,
        )),
    ];

    world
}

fn two_perline_spheres() -> Vec<Box<dyn Hittable>> {
    let material_ground = Arc::new(Lambertian::with_texture(Box::new(NoiseTexture::new(4.0))));
    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::steady(
            Vec3::f32(0.0, -1000., 0.0),
            1000.0,
            material_ground.clone(),
        )),
        Box::new(Sphere::steady(
            Vec3::f32(0.0, 2., 0.0),
            2.0,
            material_ground,
        )),
    ];

    world
}

fn earth() -> Vec<Box<dyn Hittable>> {
    let path = env::current_dir().unwrap().join("earthmap.jpg");
    let earth_texture = Box::new(ImageTexture::new(path));
    let earth_surface = Arc::new(Lambertian::with_texture(earth_texture));
    let globe = Box::new(Sphere::steady(
        Point::f32(0.0, 0.0, 0.0),
        2.0,
        earth_surface,
    ));

    vec![globe]
}

fn simple_light() -> Vec<Box<dyn Hittable>> {
    let material_ground = Arc::new(Lambertian::with_texture(Box::new(NoiseTexture::new(4.0))));
    let world: Vec<Box<dyn Hittable>> = vec![
        Box::new(Sphere::steady(
            Vec3::f32(0.0, -1000., 0.0),
            1000.0,
            material_ground.clone(),
        )),
        Box::new(Sphere::steady(
            Vec3::f32(0.0, 2., 0.0),
            2.0,
            material_ground,
        )),
        Box::new(XyRectangle::new(
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

fn cornell_box() -> Vec<Box<dyn Hittable>> {
    let red = Arc::new(Lambertian::with_color(Color::f32(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::with_color(Color::f32(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::with_color(Color::f32(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::f32(15.0, 15.0, 15.0)));

    vec![
        Box::new(YzRectangle::new(0.0, 555.0, 0.0, 555.0, 555.0, green)),
        Box::new(YzRectangle::new(0.0, 555.0, 0.0, 555.0, 0.0, red)),
        Box::new(XzRectangle::new(213.0, 343.0, 227.0, 332.0, 554.0, light)),
        Box::new(XzRectangle::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())),
        Box::new(XzRectangle::new(
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        )),
        Box::new(XyRectangle::new(
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        )),
        Box::new(Translate::new(
            Box::new(RotateY::new(
                Box::new(Boxes::new(
                    &Point::f32(0.0, 0.0, 0.0),
                    &Point::f32(165.0, 330.0, 165.0),
                    white.clone(),
                )),
                15.0,
            )),
            Vec3::f32(265.0, 0.0, 295.0),
        )),
        Box::new(Translate::new(
            Box::new(RotateY::new(
                Box::new(Boxes::new(
                    &Point::f32(0.0, 0.0, 0.0),
                    &Point::f32(165.0, 165.0, 165.0),
                    white,
                )),
                -18.0,
            )),
            Vec3::f32(130.0, 0.0, 65.0),
        )),
    ]
}

fn main() {
    // World And Camera
    let case = 6;
    let (world, look_from, look_at, aperture, vfov, background, aspect_ration) = match case {
        1 => (
            two_spheres(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
            16.0 / 9.0,
        ),
        2 => (
            two_perline_spheres(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
            16.0 / 9.0,
        ),
        3 => (
            earth(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
            16.0 / 9.0,
        ),
        4 => (
            random_scene(),
            Vec3::f32(13.0, 2.0, 3.0),
            Vec3::f32(0.0, 0.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.70, 0.80, 1.00),
            16.0 / 9.0,
        ),
        5 => (
            simple_light(),
            Vec3::f32(26.0, 3.0, 6.0),
            Vec3::f32(0.0, 2.0, 0.0),
            0.1,
            20.0,
            Color::f32(0.0, 0.0, 0.00),
            16.0 / 9.0,
        ),
        6 => (
            cornell_box(),
            Vec3::f32(278.0, 278.0, -800.0),
            Vec3::f32(278.0, 278.0, 0.0),
            0.1,
            40.0,
            Color::f32(0.0, 0.0, 0.00),
            1.0,
        ),
        _ => (
            vec![],
            Vec3::default(),
            Vec3::default(),
            0.0,
            0.0,
            Color::default(),
            16.0 / 9.0,
        ),
    };

    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ration) as i32;
    let samples_per_pixel = 300;
    let max_depth = 16;

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

    let colors = (0..image_height + 1)
        .into_par_iter()
        .rev()
        .map(|j| {
            println!("Scan line:{}", j);
            (0..image_width)
                .into_par_iter()
                .map(|i| {
                    (0..samples_per_pixel)
                        .into_par_iter()
                        .map(|_| {
                            let mut range = thread_rng();
                            let u =
                                (i as f32 + range.gen_range(0.0..1.0)) / (image_width - 1) as f32;
                            let v =
                                (j as f32 + range.gen_range(0.0..1.0)) / (image_height - 1) as f32;
                            let r = camera.get_ray(u, v);
                            ray_color(&r, &background, &world, max_depth)
                        })
                        .sum()
                })
                .collect()
        })
        .collect::<Image>();
    // Render
    let current_dir = env::current_dir().unwrap();
    let mut file = File::create(current_dir.join(format!("{}.ppm", case))).unwrap();

    file.write_all(format!("P3\n{} {}\n255\n", image_width, image_height).as_bytes())
        .unwrap();
    write_colors(&mut file, &colors, samples_per_pixel);
}

fn write_colors(f: &mut File, pixel_colors: &Image, samples_per_pixel: i32) {
    let scale = 1.0 / samples_per_pixel as f32;
    for pixel_colors in pixel_colors {
        for pixel_color in pixel_colors {
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
    }
}
