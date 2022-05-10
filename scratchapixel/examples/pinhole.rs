use std::env;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;

use scratchapixel::matrix::Matrix44;
use scratchapixel::vec::{Vec2, Vec3};

const FOCAL_LENGTH: f32 = 35.0;
const FILM_APERTURE_WIDTH: f32 = 0.825;
const FILM_APERTURE_HEIGHT: f32 = 0.446;
const INCH_TO_MM: f32 = 25.4;
const NEAR_CLIPPING_PLANE: f32 = 0.1;
const FAR_CLIPPING_PLACE: f32 = 1000.0;

const IMAGE_WIDTH: usize = 512;
const IMAGE_HEIGHT: usize = 512;

enum FitResolutionGate {
    Fill,
    Overscan,
}

const TRIS: usize = 51;
const TRIS_IDX: [usize; TRIS * 3] = [
    4, 0, 5, 0, 1, 5, 1, 2, 5, 5, 2, 6, 3, 7, 2, 2, 7, 6, 5, 9, 4, 4, 9, 8, 5, 6, 9, 9, 6, 10, 7,
    11, 6, 6, 11, 10, 9, 13, 8, 8, 13, 12, 10, 14, 9, 9, 14, 13, 10, 11, 14, 14, 11, 15, 17, 16,
    13, 12, 13, 16, 13, 14, 17, 17, 14, 18, 15, 19, 14, 14, 19, 18, 16, 17, 20, 20, 17, 21, 18, 22,
    17, 17, 22, 21, 18, 19, 22, 22, 19, 23, 20, 21, 0, 21, 1, 0, 22, 2, 21, 21, 2, 1, 22, 23, 2, 2,
    23, 3, 3, 23, 24, 3, 24, 7, 24, 23, 15, 15, 23, 19, 24, 15, 7, 7, 15, 11, 0, 25, 20, 0, 4, 25,
    20, 25, 16, 16, 25, 12, 25, 4, 12, 12, 4, 8, 26, 27, 28, 29, 30, 31, 32, 34, 33,
];

/// thanks [`scratchapixel`]
/// This code is copy from ['pinhole'] and translate to rust
///
/// [`scratchapixel`]: https://www.scratchapixel.com/index.php/
/// [`pinhole`]: https://www.scratchapixel.com/code.php?id=24&origin=/lessons/3d-basic-rendering/3d-viewing-pinhole-camera
fn main() {
    let verts = [
        Vec3::f32(-2.5703, 0.78053, -2.4e-05),
        Vec3::f32(-0.89264, 0.022582, 0.018577),
        Vec3::f32(1.6878, -0.017131, 0.022032),
        Vec3::f32(3.4659, 0.025667, 0.018577),
        Vec3::f32(-2.5703, 0.78969, -0.001202),
        Vec3::f32(-0.89264, 0.25121, 0.93573),
        Vec3::f32(1.6878, 0.25121, 1.1097),
        Vec3::f32(3.5031, 0.25293, 0.93573),
        Vec3::f32(-2.5703, 1.0558, -0.001347),
        Vec3::f32(-0.89264, 1.0558, 1.0487),
        Vec3::f32(1.6878, 1.0558, 1.2437),
        Vec3::f32(3.6342, 1.0527, 1.0487),
        Vec3::f32(-2.5703, 1.0558, 0.0),
        Vec3::f32(-0.89264, 1.0558, 0.0),
        Vec3::f32(1.6878, 1.0558, 0.0),
        Vec3::f32(3.6342, 1.0527, 0.0),
        Vec3::f32(-2.5703, 1.0558, 0.001347),
        Vec3::f32(-0.89264, 1.0558, -1.0487),
        Vec3::f32(1.6878, 1.0558, -1.2437),
        Vec3::f32(3.6342, 1.0527, -1.0487),
        Vec3::f32(-2.5703, 0.78969, 0.001202),
        Vec3::f32(-0.89264, 0.25121, -0.93573),
        Vec3::f32(1.6878, 0.25121, -1.1097),
        Vec3::f32(3.5031, 0.25293, -0.93573),
        Vec3::f32(3.5031, 0.25293, 0.0),
        Vec3::f32(-2.5703, 0.78969, 0.0),
        Vec3::f32(1.1091, 1.2179, 0.0),
        Vec3::f32(1.145, 6.617, 0.0),
        Vec3::f32(4.0878, 1.2383, 0.0),
        Vec3::f32(-2.5693, 1.1771, -0.081683),
        Vec3::f32(0.98353, 6.4948, -0.081683),
        Vec3::f32(-0.72112, 1.1364, -0.081683),
        Vec3::f32(0.9297, 6.454, 0.0),
        Vec3::f32(-0.7929, 1.279, 0.0),
        Vec3::f32(0.91176, 1.2994, 0.0),
    ];

    let film_aspect_ration = FILM_APERTURE_WIDTH / FILM_APERTURE_HEIGHT;
    let device_aspect_ration = IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32;

    // only handle overscan first
    let (x_scale, y_scale) = if film_aspect_ration > device_aspect_ration {
        (1.0f32, film_aspect_ration / device_aspect_ration)
    } else {
        (device_aspect_ration / film_aspect_ration, 1.0f32)
    };

    let top =
        (FILM_APERTURE_HEIGHT * INCH_TO_MM / 2.0 / FOCAL_LENGTH * NEAR_CLIPPING_PLANE) * y_scale;
    let right =
        (FILM_APERTURE_WIDTH * INCH_TO_MM / 2.0 / FOCAL_LENGTH * NEAR_CLIPPING_PLANE) * x_scale;

    let bottom = -top;
    let left = -right;

    println!(
        "Screen window coordinates: {:?} {:?} {:?} {:?}",
        bottom, left, top, right
    );
    println!(
        "Film Aspect Ration: {:?}\nDevice Aspect Ratio: {:?}",
        film_aspect_ration, device_aspect_ration
    );
    println!(
        "Angle of view: {:?} (deg)",
        2.0 * (FILM_APERTURE_WIDTH * INCH_TO_MM / 2.0 / FOCAL_LENGTH).atan() * 180.0 / PI
    );

    let current_dir = env::current_dir().unwrap();
    let mut file = File::create(current_dir.join("pinhole.svg")).unwrap();

    file.write_all(format!("<svg version=\"1.1\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">", IMAGE_WIDTH, IMAGE_HEIGHT).as_bytes()).unwrap();

    let camera_to_world: Matrix44<f32> = Matrix44 {
        m: [
            [-0.95424, 0.0, 0.299041, 0.0],
            [0.0861242, 0.95763, 0.274823, 0.0],
            [-0.28637, 0.288002, -0.913809, 0.0],
            [-3.734612, 7.610426, -14.152769, 1.0],
        ],
    };
    let world_to_camera = camera_to_world.inverse();

    let mut buffer = String::new();
    for i in 0..TRIS {
        let v0_world = &verts[TRIS_IDX[i * 3]];
        let v1_world = &verts[TRIS_IDX[i * 3 + 1]];
        let v2_world = &verts[TRIS_IDX[i * 3 + 2]];

        let (mut v0_raster, mut v1_raster, mut v2_raster) = (
            Vec2::<usize>::default(),
            Vec2::<usize>::default(),
            Vec2::<usize>::default(),
        );

        let visible = true;

        buffer.clear();
        let val = if visible {
            0
        } else {
            255
        };

        compute_pixel_coordinates(
            &v0_world,
            &world_to_camera,
            bottom,
            left,top,right,
            NEAR_CLIPPING_PLANE,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            &mut v0_raster,
        );
        compute_pixel_coordinates(
            &v1_world,
            &world_to_camera,
            bottom,
            left,top,right,
            NEAR_CLIPPING_PLANE,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            &mut v1_raster,
        );
        compute_pixel_coordinates(
            &v2_world,
            &world_to_camera,
            bottom,
            left,top,right,
            NEAR_CLIPPING_PLANE,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            &mut v2_raster,
        );
        buffer += &format!("<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" style=\"stroke:rgb({},0,0);stroke-width:1\" />\n", v0_raster.x, v0_raster.y, v1_raster.x, v1_raster.y, val);
        buffer += &format!("<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" style=\"stroke:rgb({},0,0);stroke-width:1\" />\n", v1_raster.x, v1_raster.y, v2_raster.x, v2_raster.y, val);
        buffer += &format!("<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" style=\"stroke:rgb({},0,0);stroke-width:1\" />\n", v2_raster.x, v2_raster.y, v0_raster.x, v0_raster.y, val);

        file.write_all(buffer.as_bytes()).unwrap();
    }

    file.write_all("</svg>\n".as_bytes()).unwrap();
    file.flush().unwrap();

}

fn compute_pixel_coordinates(
    p_word: &Vec3<f32>,
    world_to_camera: &Matrix44<f32>,
    b: f32,
    l: f32,
    t: f32,
    r: f32,
    near: f32,
    image_width: usize,
    image_height: usize,
    p_raster: &mut Vec2<usize>,
) -> bool {

    let mut p_camera = Vec3::<f32>::default();
    world_to_camera.mult_vec_matrix(p_word, &mut p_camera);
    let p_screen = Vec2 {
        x: p_camera.x / -p_camera.z * near,
        y: p_camera.y / -p_camera.z * near,
    };

    let p_ndc = Vec2 {
        x: (p_screen.x + r) / (2.0 * r),
        y: (p_screen.y + t) / (2.0 * r),
    };

    p_raster.x = (p_ndc.x * image_width as f32) as usize;
    p_raster.y =  ((1.0 - p_ndc.y) * image_height as f32) as usize;

    let  visible = if p_screen.x < l || p_screen.x > r || p_screen.y < b || p_screen.y > t {
        false
    } else {
        true
    };

    visible
}
