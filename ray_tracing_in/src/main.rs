use rand::{thread_rng, Rng};
use ray_tracing_in::camera::Camera;
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

use ray_tracing_in::bvh::BvhNode;
use ray_tracing_in::hittable::{
    ConstantMedium, FlipNormals, Hittable, Isotropic, RotateY, Translate,
};
use ray_tracing_in::material::{Dielectric, DiffuseLight, Lambertian, Metal};
use ray_tracing_in::ray::Ray;
use ray_tracing_in::rectangle::{AARect, Plane};
use ray_tracing_in::sphere::Sphere;
use ray_tracing_in::texture::{CheckerTexture, ImageTexture, NoiseTexture, SolidColor};
use ray_tracing_in::vec::Vec3;
use ray_tracing_in::{clamp, Cube, Point};
use ray_tracing_in::{Color, Image};

fn ray_color(r: &Ray, background: &Color, world: &[Box<dyn Hittable>], depth: i32) -> Color {
    if let Some(hit) = world.hit(r, 0.001, f32::MAX) {
        let emitted = hit.material.emitted(hit.u, hit.v, &hit.p);
        if depth < 50 {
            if let Some((attenuation, scattered)) = hit.material.scatter(r, &hit) {
                return emitted + attenuation * ray_color(&scattered, background, world, depth + 1);
            }
        }
        emitted
    } else {
        *background
    }
}

fn random_scene() -> Vec<Box<dyn Hittable>> {
    let mut world: Vec<Box<dyn Hittable>> = vec![Box::new(Sphere::steady(
        Vec3::f32(0.0, -1000., 0.0),
        1000.0,
        Lambertian::new(CheckerTexture::new(
            SolidColor::new(0.2, 0.3, 0.1),
            SolidColor::new(0.9, 0.9, 0.9),
        )),
    ))];

    let mut rang = thread_rng();
    let point = Vec3::f32(4.0, 0.2, 0.0);
    let mut smalls: Vec<Arc<dyn Hittable>> = vec![];
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
                    smalls.push(Arc::new(Sphere::motion(
                        center,
                        center2,
                        0.2,
                        0.0,
                        1.0,
                        Lambertian::new(SolidColor { color: albedo }),
                    )));
                } else if choose_mat < 0.95 {
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = rang.gen_range(0.0..0.5);
                    smalls.push(Arc::new(Sphere::steady(
                        center,
                        0.2,
                        Metal::new(albedo, fuzz),
                    )));
                } else {
                    smalls.push(Arc::new(Sphere::steady(center, 0.2, Dielectric::new(1.5))));
                }
            }
        }
    }

    world.push(Box::new(BvhNode::new(&mut smalls, 0.0, 1.0)));

    world.push(Box::new(Sphere::steady(
        Vec3::f32(0.0, 1.0, 0.0),
        1.0,
        Dielectric::new(1.5),
    )));
    world.push(Box::new(Sphere::steady(
        Vec3::f32(-4.0, 1.0, 0.0),
        1.0,
        Lambertian::new(SolidColor::new(0.4, 0.2, 0.1)),
    )));
    world.push(Box::new(Sphere::steady(
        Vec3::f32(4.0, 1.0, 0.0),
        1.0,
        Metal::new(Color::f32(0.7, 0.6, 0.5), 0.0),
    )));

    world
}

fn two_spheres() -> Vec<Box<dyn Hittable>> {
    let material_ground = Lambertian::new(CheckerTexture::new(
        SolidColor::new(0.2, 0.3, 0.1),
        SolidColor::new(0.9, 0.9, 0.9),
    ));
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
    let material_ground = Lambertian::new(NoiseTexture::new(4.0));
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
    let earth_texture = ImageTexture::new(path);
    let earth_surface = Lambertian::new(earth_texture);
    let globe = Box::new(Sphere::steady(
        Point::f32(0.0, 0.0, 0.0),
        2.0,
        earth_surface,
    ));

    vec![globe]
}

fn simple_light() -> Vec<Box<dyn Hittable>> {
    let material_ground = Lambertian::new(CheckerTexture::new(
        SolidColor::new(0.0, 0.0, 0.0),
        SolidColor::new(1.0, 1.0, 1.0),
    ));
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
        // Box::new(AARect::new(
        //     Plane::XY,
        //     3.0,
        //     5.0,
        //     1.0,
        //     3.0,
        //     -2.0,
        //     DiffuseLight::new(SolidColor::new(4., 4., 4.)),
        // )),
    ];

    world
}

fn cornell_box() -> Vec<Box<dyn Hittable>> {
    let red = Lambertian::new(SolidColor::new(0.65, 0.05, 0.05));
    let white = Lambertian::new(SolidColor::new(0.73, 0.73, 0.73));
    let green = Lambertian::new(SolidColor::new(0.12, 0.45, 0.15));
    let light = DiffuseLight::new(SolidColor::new(15.0, 15.0, 15.0));

    vec![
        Box::new(FlipNormals(AARect::new(
            Plane::YZ,
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            green,
        ))),
        Box::new(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, red)),
        Box::new(AARect::new(
            Plane::XZ,
            213.0,
            343.0,
            227.0,
            332.0,
            554.0,
            light,
        )),
        Box::new(AARect::new(
            Plane::XZ,
            0.0,
            555.0,
            0.0,
            555.0,
            0.0,
            white.clone(),
        )),
        Box::new(FlipNormals(AARect::new(
            Plane::XZ,
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        ))),
        Box::new(FlipNormals(AARect::new(
            Plane::XY,
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        ))),
        Box::new(Translate::new(
            RotateY::new(
                Cube::new(
                    &Point::f32(0.0, 0.0, 0.0),
                    &Point::f32(165.0, 330.0, 165.0),
                    white.clone(),
                ),
                15.0,
            ),
            Vec3::f32(265.0, 0.0, 295.0),
        )),
        Box::new(Translate::new(
            RotateY::new(
                Cube::new(
                    &Point::f32(0.0, 0.0, 0.0),
                    &Point::f32(165.0, 165.0, 165.0),
                    white,
                ),
                -18.0,
            ),
            Vec3::f32(130.0, 0.0, 65.0),
        )),
    ]
}

fn cornell_smoke() -> Vec<Box<dyn Hittable>> {
    let red = Lambertian::new(SolidColor::new(0.65, 0.05, 0.05));
    let white = Lambertian::new(SolidColor::new(0.73, 0.73, 0.73));
    let green = Lambertian::new(SolidColor::new(0.12, 0.45, 0.15));
    let light = DiffuseLight::new(SolidColor::new(7.0, 7.0, 7.0));

    vec![
        Box::new(FlipNormals(AARect::new(
            Plane::YZ,
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            green,
        ))),
        Box::new(AARect::new(Plane::YZ, 0.0, 555.0, 0.0, 555.0, 0.0, red)),
        Box::new(AARect::new(
            Plane::XZ,
            127.0,
            432.0,
            113.0,
            443.0,
            554.0,
            light,
        )),
        Box::new(AARect::new(
            Plane::XZ,
            0.0,
            555.0,
            0.0,
            555.0,
            0.0,
            white.clone(),
        )),
        Box::new(FlipNormals(AARect::new(
            Plane::XZ,
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        ))),
        Box::new(FlipNormals(AARect::new(
            Plane::XY,
            0.0,
            555.0,
            0.0,
            555.0,
            555.0,
            white.clone(),
        ))),
        Box::new(ConstantMedium::new(
            Translate::new(
                RotateY::new(
                    Cube::new(
                        &Point::f32(0.0, 0.0, 0.0),
                        &Point::f32(165.0, 330.0, 165.0),
                        white.clone(),
                    ),
                    15.0,
                ),
                Vec3::f32(265.0, 0.0, 295.0),
            ),
            0.01,
            Isotropic::new(SolidColor::new(0.0, 0.0, 0.0)),
        )),
        Box::new(ConstantMedium::new(
            Translate::new(
                RotateY::new(
                    Cube::new(
                        &Point::f32(0.0, 0.0, 0.0),
                        &Point::f32(165.0, 165.0, 165.0),
                        white,
                    ),
                    -18.0,
                ),
                Vec3::f32(130.0, 0.0, 65.0),
            ),
            0.01,
            Isotropic::new(SolidColor::new(1.0, 1.0, 1.0)),
        )),
    ]
}

fn final_scene() -> Vec<Box<dyn Hittable>> {
    let mut objects: Vec<Box<dyn Hittable>> = vec![Box::new(AARect::new(
        Plane::XZ,
        123.0,
        423.0,
        147.0,
        412.0,
        554.0,
        DiffuseLight::new(SolidColor::new(7.0, 7.0, 7.0)),
    ))];

    let mut rang = rand::thread_rng();
    let ground = Lambertian::new(SolidColor::new(0.48, 0.83, 0.53));
    let boxes_per_side = 20;
    let mut boxes1: Vec<Arc<dyn Hittable>> = vec![];
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            const W: f32 = 100.0;
            let x0 = -1000.0 + i as f32 * W;
            let y0 = 0.0;
            let z0 = -1000.0 + j as f32 * W;
            let x1 = x0 + W;
            let y1 = 100.0 * (rang.gen::<f32>() + 0.01);
            let z1 = z0 + W;

            boxes1.push(Arc::new(Cube::new(
                &Point::f32(x0, y0, z0),
                &Point::f32(x1, y1, z1),
                ground.clone(),
            )));
        }
    }
    objects.push(Box::new(BvhNode::new(&mut boxes1, 0.0, 1.0)));

    let path = env::current_dir().unwrap().join("earthmap.jpg");
    let earth_surface = Lambertian::new(ImageTexture::new(path));
    objects.push(Box::new(Sphere::steady(
        Point::f32(400.0, 200.0, 400.0),
        100.0,
        earth_surface,
    )));

    objects.push(Box::new(Sphere::steady(
        Point::f32(220.0, 280.0, 300.0),
        80.0,
        Lambertian::new(NoiseTexture::new(0.1)),
    )));

    let center1 = Point::f32(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::f32(30.0, 0.0, 0.0);
    let moving_sphere_material = Lambertian::new(SolidColor::new(0.7, 0.3, 0.1));
    objects.push(Box::new(Sphere::motion(
        center1,
        center2,
        50.0,
        0.0,
        1.0,
        moving_sphere_material,
    )));

    objects.push(Box::new(Sphere::steady(
        Point::f32(260.0, 150.0, 45.0),
        50.0,
        Dielectric::new(1.5),
    )));
    objects.push(Box::new(Sphere::steady(
        Point::f32(0.0, 150.0, 145.0),
        50.0,
        Metal::new(Color::f32(0.8, 0.8, 0.9), 1.0),
    )));

    objects.push(Box::new(Sphere::steady(
        Point::f32(360.0, 150.0, 145.0),
        70.0,
        Dielectric::new(1.5),
    )));
    objects.push(Box::new(ConstantMedium::new(
        Sphere::steady(Point::f32(360.0, 150.0, 145.0), 70.0, Dielectric::new(1.5)),
        0.2,
        Isotropic::new(SolidColor::new(0.2, 0.4, 0.9)),
    )));

    let mut boxes: Vec<Arc<dyn Hittable>> = vec![];
    let white = Lambertian::new(SolidColor::new(0.73, 0.732, 0.73));
    for _ in 0..1000 {
        boxes.push(Arc::new(Sphere::steady(
            Point::random_range(0.0, 165.0),
            10.0,
            white.clone(),
        )));
    }

    objects.push(Box::new(Translate::new(
        RotateY::new(BvhNode::new(&mut boxes, 0.0, 1.0), 15.0),
        Vec3::f32(-100.0, 270.0, 395.0),
    )));

    objects.push(Box::new(ConstantMedium::new(
        Sphere::steady(Point::f32(0.0, 0.0, 0.0), 5000.0, Dielectric::new(1.5)),
        0.0001,
        Isotropic::new(SolidColor::new(1.0, 1.0, 1.0)),
    )));

    objects
}

fn main() {
    // World And Camera
    let case = 4;
    let (world, look_from, look_at, aperture, vfov, background, aspect_ration, samples_per_pixel) =
        match case {
            1 => (
                two_spheres(),
                Vec3::f32(13.0, 2.0, 3.0),
                Vec3::f32(0.0, 0.0, 0.0),
                0.1,
                20.0,
                Color::f32(0.70, 0.80, 1.00),
                16.0 / 9.0,
                300,
            ),
            2 => (
                two_perline_spheres(),
                Vec3::f32(13.0, 2.0, 3.0),
                Vec3::f32(0.0, 0.0, 0.0),
                0.1,
                20.0,
                Color::f32(0.70, 0.80, 1.00),
                16.0 / 9.0,
                300,
            ),
            3 => (
                earth(),
                Vec3::f32(13.0, 2.0, 3.0),
                Vec3::f32(0.0, 0.0, 0.0),
                0.1,
                20.0,
                Color::f32(0.70, 0.80, 1.00),
                16.0 / 9.0,
                300,
            ),
            4 => (
                random_scene(),
                Vec3::f32(13.0, 2.0, 3.0),
                Vec3::f32(0.0, 0.0, 0.0),
                0.1,
                20.0,
                Color::f32(0.70, 0.80, 1.00),
                16.0 / 9.0,
                500,
            ),
            5 => (
                simple_light(),
                Vec3::f32(26.0, 3.0, 6.0),
                Vec3::f32(0.0, 2.0, 0.0),
                0.1,
                20.0,
                Color::f32(0.70, 0.80, 1.00),
                16.0 / 9.0,
                800,
            ),
            6 => (
                cornell_box(),
                Vec3::f32(278.0, 278.0, -800.0),
                Vec3::f32(278.0, 278.0, 0.0),
                0.1,
                40.0,
                Color::f32(0.0, 0.0, 0.00),
                1.0,
                300,
            ),
            7 => (
                cornell_smoke(),
                Vec3::f32(278.0, 278.0, -800.0),
                Vec3::f32(278.0, 278.0, 0.0),
                0.1,
                40.0,
                Color::f32(0.0, 0.0, 0.00),
                1.0,
                300,
            ),
            8 => (
                final_scene(),
                Vec3::f32(478.0, 278.0, -600.0),
                Vec3::f32(278.0, 278.0, 0.0),
                0.0,
                40.0,
                Color::f32(0.0, 0.0, 0.00),
                16.0 / 9.0,
                1280,
            ),
            _ => (
                vec![],
                Vec3::default(),
                Vec3::default(),
                0.0,
                0.0,
                Color::default(),
                16.0 / 9.0,
                1,
            ),
        };

    let image_width = 1280;
    let image_height = (image_width as f32 / aspect_ration) as i32;

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

    let instant = Instant::now();
    let mut buffers: Image = Vec::with_capacity(image_height as usize);

    (0..image_height + 1)
        .into_par_iter()
        .rev()
        .map(|j| {
            (0..image_width)
                .into_par_iter()
                .map(|i| {
                    (0..samples_per_pixel)
                        .into_par_iter()
                        .map(|_| {
                            let u =
                                (i as f32 + thread_rng().gen::<f32>()) / (image_width - 1) as f32;
                            let v =
                                (j as f32 + thread_rng().gen::<f32>()) / (image_height - 1) as f32;
                            let r = camera.get_ray(u, v);
                            ray_color(&r, &background, &world, 0)
                        })
                        .sum()
                })
                .collect()
        })
        .collect_into_vec(&mut buffers);

    println!("Took {:?} wall time", instant.elapsed());
    // Render
    let current_dir = env::current_dir().unwrap();
    let mut file = File::create(current_dir.join(format!("{}.ppm", case))).unwrap();
    file.write_all(format!("P3\n{} {}\n255\n", image_width, image_height).as_bytes())
        .unwrap();
    write_colors(&mut file, &buffers, samples_per_pixel);
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
                    (256.0 * clamp(r, 0.0, 0.999)) as u8,
                    (256.0 * clamp(g, 0.0, 0.999)) as u8,
                    (256.0 * clamp(b, 0.0, 0.999)) as u8,
                )
                .as_bytes(),
            )
            .unwrap();
        }
    }
}
