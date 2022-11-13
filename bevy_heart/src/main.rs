use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};
use rand::{rngs::ThreadRng, thread_rng, Rng, distributions::Uniform, prelude::Distribution};
use std::f32::consts::PI;
use std::f32::consts::E;

const IMAGE_ENLARGE: f32 = 16.0;
const HALO_FRAMES: i32 = 20;

struct HeartPoints {
    points: Vec<Vec3>,
    edge_diff: Vec<Vec3>,
    center_diff: Vec<Vec3>,
    halo: Vec<Vec<Vec3>>,
}
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PointType {
    EDGE,
    EdgeDiff,
    CenterDiff,
    Halo,
}
#[derive(Component)]
pub struct Vec3Wrapper(Vec3);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(movement)
        .add_system_set(
            SystemSet::new()
                //控制下光环的刷新频率
                .with_run_criteria(FixedTimestep::step(3.0 / 60.0))
                .with_system(movement_halo),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());
    // Circle
    let mut rnd = thread_rng();

    let mut points = vec![];

    let between = Uniform::from(0.0..2.0 * PI);
    for _ in 0..rnd.gen_range(100..1000) {
        let t = between.sample(&mut rnd);
        let (x, y) = heart_function(t, IMAGE_ENLARGE);
        points.push(Vec3::new(x, y, 0.));
    }

    // 爱心向内扩撒
    let mut edge_diffusion_points = vec![];
    for v in &points {
        for _ in 1..3 {
            let (x, y) = scatter_inside(v.x, v.y, 0.05, &mut rnd);
            edge_diffusion_points.push(Vec3::new(x, y, 0.0));
        }
    }

    let mut center_diffusion_points = vec![];
    for _ in 0..rnd.gen_range(600..1500) {
        let v = &points[rnd.gen_range(0..points.len())];
        let (x, y) = scatter_inside(v.x, v.y, 0.3, &mut rnd);
        center_diffusion_points.push(Vec3::new(x, y, 0.0));
    }

    let heart_points = HeartPoints {
        points,
        edge_diff: edge_diffusion_points,
        center_diff: center_diffusion_points,
        halo: halo_point(&mut rnd),
    };
    draw_points(
        &heart_points.points,
        &mut commands,
        &mut meshes,
        &mut materials,
        PointType::EDGE,
    );
    draw_points(
        &heart_points.edge_diff,
        &mut commands,
        &mut meshes,
        &mut materials,
        PointType::EdgeDiff,
    );
    draw_points(
        &heart_points.center_diff,
        &mut commands,
        &mut meshes,
        &mut materials,
        PointType::CenterDiff,
    );

    draw_points(
        &heart_points.halo[0],
        &mut commands,
        &mut meshes,
        &mut materials,
        PointType::Halo,
    );

    commands.insert_resource(0);
    commands.insert_resource(heart_points);
}

fn halo_point(rand: &mut ThreadRng) -> Vec<Vec<Vec3>> {
    let frames = HALO_FRAMES;
    let mut halo_points: Vec<Vec<Vec3>> = vec![];

    let between = Uniform::from(0.0..2.0 * PI);
    let mut set = vec![];
    for generate_frame in 0..frames {
        let halo_radius = 3.0 + 4.0 * (1.0 + curve(generate_frame as f32 / 10.0 * PI));
        let halo_number =
            (2000.0 + 4000.0 * curve(generate_frame as f32 / 10.0 * PI).powi(2).abs()) as i32;
        let mut heart_halo_point = vec![];
        for _ in 0..rand.gen_range(300..halo_number) {
            let t = between.sample(rand); // 随机不到的地方造成爱心有缺口
            let (x, y) = heart_function(t, IMAGE_ENLARGE + 0.6); // 魔法参数
            let (x, y) = shrink(x, y, halo_radius, rand);
            let v = Vec3::new(x, y, 0.0);
            if !set.contains(&v) {
                // 处理新的点
                set.push(v);
                let x = x + rand.gen_range(-14.0..14.0);
                let y = y + rand.gen_range(-14.0..14.0);
                heart_halo_point.push(Vec3::new(x, y, 0.0));
            }
        }

        halo_points.push(heart_halo_point);
    }

    halo_points
}

fn draw_points(
    vec3s: &[Vec3],
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    point_type: PointType,
) {
    let color = ColorMaterial::from(Color::RED);
    for (idx, v) in vec3s.iter().enumerate() {
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(1.).into()).into(),
                material: materials.add(color.clone()),
                transform: Transform::from_translation(*v),
                ..default()
            })
            .insert(point_type.clone())
            .insert(if point_type == PointType::Halo {
                Vec3Wrapper(Vec3::new(idx as f32, 0.0, 0.0))
            } else {
                Vec3Wrapper(*v)
            });
    }
}

fn movement(
    mut query: Query<(&PointType, &Vec3Wrapper, &mut Transform)>,
    mut frame_idx: ResMut<i32>,
) {
    //魔法数，调整跳动的频率
    *frame_idx += 1;
    let idx = frame_idx.as_ref() % 100;
    let ration = curve(idx as f32 / 100.0 * PI) * 15.0;
    let mut rnd = thread_rng();
    for (pt, v, mut trans) in query.iter_mut() {
        match *pt {
            PointType::EdgeDiff | PointType::CenterDiff => {
                let (x, y) = (v.0.x, v.0.y);
                let (x, y) = calc_position(x, y, ration, &mut rnd);
                trans.translation.x = x;
                trans.translation.y = y;
                trans.scale = Vec3::ONE * rnd.gen_range(0.9..1.2);
            }
            PointType::EDGE => {
                let (x, y) = (v.0.x, v.0.y);
                let (x, y) = calc_position(x, y, ration, &mut rnd);
                trans.translation.x = x;
                trans.translation.y = y;
                trans.scale = Vec3::ONE * rnd.gen_range(1.0..1.3);
            }
            _ => {}
        }
    }
}

fn movement_halo(
    mut query: Query<(&PointType, &Vec3Wrapper, &mut Transform)>,
    frame_idx: Res<i32>,
    heart_points: Res<HeartPoints>,
) {
    //魔法数，调整跳动的频率
    let halo_points = frame_idx.as_ref() % 20;
    let halo_points = &heart_points.halo[halo_points as usize];
    let mut rnd = thread_rng();
    for (pt, v, mut trans) in query.iter_mut() {
        match *pt {
            PointType::Halo => {
                let idx = v.0.x as usize;
                if idx < halo_points.len() {
                    let v = halo_points[idx];
                    trans.translation.x = v.x;
                    trans.translation.y = v.y;
                    trans.scale = Vec3::ONE * rnd.gen_range(1.0..1.2);
                }
            }
            _ => {}
        }
    }
}

fn heart_function(t: f32, shrink_ration: f32) -> (f32, f32) {
    let mut x = 16. * t.sin().powi(3);
    let mut y = 13. * t.cos() - 5. * (2. * t).cos() - 2. * (3. * t).cos() - (4. * t).cos();

    x *= shrink_ration;
    y *= shrink_ration;

    // x += CANVAS_CENTER_X;
    // y += CANVAS_CENTER_Y;

    (x, y)
}

fn scatter_inside(x: f32, y: f32, beta: f32, rnd: &mut ThreadRng) -> (f32, f32) {
    let ration_x = -(beta * rnd.gen::<f32>().log(E));
    let ration_y = -(beta * rnd.gen::<f32>().log(E));

    let dx = ration_x * x;
    let dy = ration_y * y;
    (x - dx, y - dy)
}

// 自定义曲线函数，调整跳动周期
// :param p: 参数
//  :return: 正弦
// 可以尝试换其他的动态函数，达到更有力量的效果（贝塞尔？）
fn curve(p: f32) -> f32 {
    2.0 * (2.0 * (4.0 * p).sin()) / (2.0 * PI)
}

fn calc_position(x: f32, y: f32, ration: f32, rand: &mut ThreadRng) -> (f32, f32) {
    // 调整缩放比例
    let force = 1.0 / (x.powi(2) + y.powi(2)).powf(0.520); //魔法数

    let dx = ration * force * x + rand.gen_range(-1.0..1.0f32);
    let dy = ration * force * y + rand.gen_range(-1.0..1.0f32);

    (x - dx, y - dy)
}

// 抖动
// :param x: 原x
// :param y: 原y
// :param ratio: 比例
// :return: 新坐标
fn shrink(x: f32, y: f32, ration: f32, rand: &mut ThreadRng) -> (f32, f32) {
    // 调整缩放比例
    let force = -1.0 / (x.powi(2) + y.powi(2)).powf(0.6); //不要问，问就是魔法数。看感觉个人调的

    let dx = ration * force * x + rand.gen_range(-1.0..1.0f32);
    let dy = ration * force * y + rand.gen_range(-1.0..1.0f32);

    (x - dx, y - dy)
}
