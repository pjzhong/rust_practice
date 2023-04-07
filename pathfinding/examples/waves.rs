use bevy::{
    math::IVec2,
    prelude::{App, Color, Commands, Query, ResMut},
    DefaultPlugins,
};
use bevy_ascii_terminal::{CharFormat, Terminal, TerminalBundle, TerminalPlugin};
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin};

pub const START_COLOR: Color = Color::BLUE;
pub const END_COLOR: Color = Color::GREEN;

pub const START_COLOR_1: Color = Color::ORANGE;
pub const END_COLOR_1: Color = Color::ORANGE_RED;

pub struct WavesConfig {
    pub x_offset: f64,
    pub move_speed: f64,
}

fn setup(mut commands: Commands) {
    let size = [128, 64];
    commands.spawn_bundle(TerminalBundle::new().with_size(size));
    commands.spawn_bundle(TiledCameraBundle::new().with_tile_count(size));

    commands.insert_resource(WavesConfig {
        x_offset: 0.0,
        move_speed: 0.1,
    });
}

fn draw(mut q_term: Query<&mut Terminal>, mut config: ResMut<WavesConfig>) {
    let mut term = q_term.single_mut();
    let size = term.size().as_ivec2();

    //y = A sin(Bx + C) + D
    // A值(振幅)越大，曲线更陡峭(也可以说是高度)
    // B值(周期)越大，周期越短，B值小于1大于0时，周期变长（也可以说是宽度）
    // C值（平移）不变， C为正值，曲线向左移动，为负值曲线向右移动
    // D值(上下)控制曲线上下移动
    let wave_height = 0.10;
    let wave_width = 0.2;
    let x_offset = config.x_offset;

    term.clear();

    let half = size.x / 2;
    for (idx, x) in (-half..half).enumerate() {
        let t = idx as f32 / (size.x - 2) as f32;
        let col = color_lerp(START_COLOR, END_COLOR, t);
        let fmt = CharFormat::new(col, Color::BLACK);
        let y =
            size.y as f64 * wave_height * ((half as f64 + x as f64) * wave_width + x_offset).sin();
        let pos = world_to_map(size, IVec2::new(x, y as i32));
        term.put_char_formatted(pos.to_array(), '█', fmt);
    }

    for (idx, x) in (-half..half).enumerate() {
        let t = idx as f32 / (size.x - 2) as f32;
        let col = color_lerp(START_COLOR_1, END_COLOR_1, t);
        let fmt = CharFormat::new(col, Color::BLACK);
        let y =
            size.y as f64 * wave_height * ((half as f64 + x as f64) * wave_width + x_offset).cos();
        let pos = world_to_map(size, IVec2::new(x, y as i32));
        term.put_char_formatted(pos.to_array(), '█', fmt);
    }

    config.x_offset += config.move_speed;
}

fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let t = f32::clamp(t, 0.0, 1.0);
    Color::rgba(
        a.r() + (b.r() - a.r()) * t,
        a.g() + (b.g() - a.g()) * t,
        a.b() + (b.b() - a.b()) * t,
        a.a() + (b.a() - a.a()) * t,
    )
}

fn world_to_map(size: IVec2, pos: IVec2) -> IVec2 {
    pos + size / 2
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_plugin(TiledCameraPlugin)
        .add_startup_system(setup)
        .add_system(draw)
        .run();
}
