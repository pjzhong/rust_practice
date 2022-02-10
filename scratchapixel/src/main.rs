use std::env;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;

use crate::vec::Vec3;

mod matrix;
mod vec;

const MAX_RAY_DEPTH: usize = 5;

struct Sphere {
    /// position of the sphere
    center: Vec3<f32>,
    /// surface color and emission (light)
    surface_color: Vec3<f32>,
    emission_color: Vec3<f32>,
    /// sphere radius and radius^2
    radius2: f32,
    /// surface transparency
    transparency: f32,
    /// reflectivity
    reflection: f32,
}

impl Sphere {
    pub fn new(
        center: Vec3<f32>,
        surface_color: Vec3<f32>,
        emission_color: Vec3<f32>,
        radius: f32,
        reflection: f32,
        transparency: f32,
    ) -> Self {
        Self {
            center,
            surface_color,
            emission_color,
            radius2: radius * radius,
            reflection,
            transparency,
        }
    }

    fn intersect(
        &self,
        ray_origination: &Vec3<f32>,
        ray_direction: &Vec3<f32>,
        t0: &mut f32,
        t1: &mut f32,
    ) -> bool {
        let l = self.center - *ray_origination;
        let tca = l.dot_product(ray_direction);
        if tca < 0.0 {
            return false;
        }

        let d2 = l.dot_product(&l) - tca * tca;
        if self.radius2 < d2 {
            return false;
        }

        let thc = (self.radius2 - d2).sqrt();
        *t0 = tca - thc;
        *t1 = tca + thc;

        true
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Vec3::f32(0., 0., 0.),
            surface_color: Vec3::f32(0., 0., 0.),
            emission_color: Vec3::f32(0., 0., 0.),
            radius2: 0.0,
            transparency: 0.0,
            reflection: 0.0,
        }
    }
}

fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}

fn trace(
    ray_origination: &Vec3<f32>,
    ray_direction: &Vec3<f32>,
    spheres: &[Sphere],
    depth: usize,
) -> Vec3<f32> {
    let mut tnear = f32::INFINITY;
    let mut nearest_sphere: Option<&Sphere> = None;

    for s in spheres {
        let mut t0 = f32::INFINITY;
        let mut t1 = f32::INFINITY;
        if s.intersect(ray_origination, ray_direction, &mut t0, &mut t1) {
            if t0 < 0.0 {
                t0 = t1;
            }

            if t0 < tnear {
                tnear = t0;
                nearest_sphere = Some(s);
            }
        }
    }

    // if there's no intersection return black or background color
    let nearest_sphere = match nearest_sphere {
        Some(v) => v,
        None => return Vec3::f32(2., 2., 2.),
    };

    // color of the ray/surface of the object intersected by the ray
    let mut surface_color = Vec3::<f32>::default();
    // point of intersection
    let point_intersection = *ray_origination + *ray_direction * tnear;
    // normal at the intersection point
    let mut normal_intersection_point = (point_intersection - nearest_sphere.center).normalize();

    // if the normal and the view direction are not opposite to each other
    // reverse the normal direction. That also means we are inside the sphere so set
    // the inside bool to true. Finally reverse the sign of IdotN which we want positive.
    let bias = 1e-4;
    let mut inside = false;
    if 0.0 < ray_direction.dot_product(&normal_intersection_point) {
        normal_intersection_point = -normal_intersection_point;
        inside = true;
    }
    if (0.0 < nearest_sphere.transparency || 0.0 < nearest_sphere.reflection)
        && depth < MAX_RAY_DEPTH
    {
        let fracing_ration = -ray_direction.dot_product(&normal_intersection_point);
        let fresneleffect = mix(f32::powf(1.0 - fracing_ration, 3.0), 1.0, 0.1);

        let refl_dir = (*ray_direction
            - normal_intersection_point
                * 2.0
                * ray_direction.dot_product(&normal_intersection_point))
        .normalize();

        // compute reflection direction (not need to normalize because all vectors are already normalized)
        let reflection = trace(
            &(point_intersection + normal_intersection_point * bias),
            &refl_dir,
            spheres,
            depth + 1,
        );
        let mut refraction = Vec3::<f32>::default();

        // if the sphere is also transparent compute refraction ray(transmission)
        if 0.0 < nearest_sphere.transparency {
            let ior = 1.1;
            let eta = if inside { ior } else { 1.0 / ior };
            let cosi = -normal_intersection_point.dot_product(ray_direction);
            let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
            let refrdir = (*ray_direction * eta
                + normal_intersection_point * (eta * cosi - k.sqrt()))
            .normalize();
            refraction = trace(
                &(point_intersection - normal_intersection_point * bias),
                &refrdir,
                spheres,
                depth + 1,
            )
        }

        // the result is a min of reflection and refraction (if the sphere is transparent);
        surface_color = (reflection * fresneleffect
            + refraction * (1.0 - fresneleffect) * nearest_sphere.transparency)
            * nearest_sphere.surface_color;
    } else {
        for i in 0..spheres.len() {
            if 0.0 < spheres[i].emission_color.x {
                let mut transmission = 1.0;
                let light_direction = (spheres[i].center - point_intersection).normalize();
                for (j, sphere) in spheres.iter().enumerate() {
                    if i != j {
                        let mut t0 = 0.0;
                        let mut t1 = 0.0;
                        if sphere.intersect(
                            &(point_intersection + normal_intersection_point * bias),
                            &light_direction,
                            &mut t0,
                            &mut t1,
                        ) {
                            transmission = 0.0;
                            break;
                        }
                    }
                }
                surface_color = surface_color
                    + (nearest_sphere.surface_color
                        * transmission
                        * normal_intersection_point
                            .dot_product(&light_direction)
                            .max(0.0)
                        * spheres[i].emission_color);
            }
        }
    }

    surface_color + nearest_sphere.emission_color
}

fn render(spheres: &[Sphere]) {
    const WIDTH: usize = 1280usize;
    const HEIGHT: usize = 960usize;
    let primary_ray = Vec3::<f32>::default();
    let mut image = vec![Vec3::<f32>::default(); WIDTH * HEIGHT];
    let inv_width = 1.0 / WIDTH as f32;
    let inv_height = 1.0 / HEIGHT as f32;

    let fov = 30.0;
    let aspect_ration = WIDTH as f32 / HEIGHT as f32;
    let angle = (PI * 0.5 * fov / 180.0).tan();

    // Trace rays
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspect_ration;
            let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
            let ray_dir = Vec3::f32(xx, yy, -1.0).normalize();
            image[y * WIDTH + x] = trace(&primary_ray, &ray_dir, spheres, 0);
        }
    }

    let current_dir = env::current_dir().unwrap();
    let mut file = File::create(current_dir.join("untitled.ppm")).unwrap();
    if file
        .write_all(format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).as_bytes())
        .is_err()
    {
        eprintln!("write file err");
        return;
    }

    for i in image {
        if file
            .write_all(&[
                (i.x.min(1.0) * 255.0) as u8,
                (i.y.min(1.0) * 255.0) as u8,
                (i.z.min(1.0) * 255.0) as u8,
            ])
            .is_err()
        {
            eprintln!("write color err");
            return;
        }
    }
}

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

    render(&spheres);
}
