use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    time::{Duration, Instant},
};

use bevy::{
    input::Input,
    math::{IVec2, Vec3},
    prelude::{
        App, Camera, Color, Commands, DetectChanges, EventReader, EventWriter, GlobalTransform,
        MouseButton, Query, Res, ResMut,
    },
    window::Windows,
    DefaultPlugins,
};
use bevy_ascii_terminal::{
    CharFormat, Pivot, StringFormat, Terminal, TerminalBundle, TerminalPlugin, Tile,
};
use bevy_tiled_camera::{TiledCameraBundle, TiledCameraPlugin, TiledProjection};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use pathfinding::{GridLocation, Pair, SquareGrid};

pub const START_COLOR: Color = Color::BLUE;
pub const END_COLOR: Color = Color::GREEN;
const WALL_VALUE: f32 = 0.45;
const WALL_TILE: Tile = Tile {
    glyph: '#',
    fg_color: Color::Rgba {
        red: WALL_VALUE,
        green: WALL_VALUE,
        blue: WALL_VALUE,
        alpha: 1.0,
    },
    bg_color: Color::BLACK,
};
const FLOOR_TILE: Tile = Tile {
    glyph: ' ',
    fg_color: Color::WHITE,
    bg_color: Color::BLACK,
};

const TEXT_FMT: StringFormat = StringFormat {
    fg_color: Color::YELLOW_GREEN,
    bg_color: Color::BLACK,
    pivot: Pivot::TopLeft,
};

#[derive(Default)]
struct PathingState {
    start: Option<IVec2>,
    goal: Option<IVec2>,
    path: Option<Vec<GridLocation>>,
    visited: Option<HashMap<GridLocation, i32>>,
    time: Duration,
}

impl PathingState {
    pub fn clear(&mut self) {
        self.start = None;
        self.goal = None;
        self.path = None;
        self.visited = None;
        self.time = Duration::ZERO;
    }
}

enum InputCommand {
    ToggleWall(IVec2),
    SetPath(IVec2),
}

fn setup(mut commands: Commands) {
    let size = [256, 128];
    commands.spawn_bundle(TerminalBundle::new().with_size(size));
    commands.spawn_bundle(TiledCameraBundle::new().with_tile_count(size));

    let mut graph = SquareGrid {
        width: size[0] as i32,
        height: size[1] as i32,
        walls: HashSet::new(),
        forest: HashSet::new(),
    };

    builds(&mut graph);

    commands.insert_resource(PathingState::default());
    commands.insert_resource(graph);
}

fn builds(map: &mut SquareGrid) {
    let fbm = Fbm::new()
        .set_octaves(16)
        .set_frequency(1.5)
        .set_lacunarity(3.0)
        .set_persistence(0.9);
    let plane = PlaneMapBuilder::new(&fbm)
        .set_size(map.width as usize, map.height as usize)
        .build();

    let threshold = 0.1;
    for x in 0..map.width {
        for y in 0..map.height {
            let v = plane.get_value(x as usize, y as usize);

            if threshold <= v {
              //  map.add_rect(x, y);
            }
        }
    }
}

fn draw(mut q_term: Query<&mut Terminal>, map: Res<SquareGrid>, finding: Res<PathingState>) {
    if !map.is_changed() && !finding.is_changed() {
        return;
    }
    let mut term = q_term.single_mut();

    for y in 0..map.height {
        for x in 0..map.width {
            let tile = term.get_tile_mut([x, y]);
            if map.passable(&GridLocation { x, y }) {
                *tile = FLOOR_TILE;
            } else {
                *tile = WALL_TILE;
            }
        }
    }

    if let (Some(path), Some(visited)) = (&finding.path, &finding.visited) {
        let fmt = CharFormat::new(Color::RED, Color::BLACK);
        for (p, _) in visited {
            let c = match map.passable(p) {
                true => WALL_TILE.glyph,
                false => '.',
            };
            term.put_char_formatted([p.x, p.y], c, fmt);
        }
        for (idx, p) in path.iter().enumerate() {
            let t = idx as f32 / (path.len() - 2) as f32;
            let col = color_lerp(START_COLOR, END_COLOR, t);
            let fmt = CharFormat::new(col, Color::BLACK);
            term.put_char_formatted([p.x, p.y], '█', fmt);
        }

        term.put_string_formatted(
            [0, 2],
            format!(
                "Found path in {:?}. Length {}. Visited {} nodes.",
                finding.time,
                path.len(),
                visited.len()
            )
            .as_str(),
            TEXT_FMT,
        );
    } else if let Some(visited) = &finding.visited {
        let fmt = CharFormat::new(Color::RED, Color::BLACK);
        for (p, _) in visited {
            let c = match map.passable(p) {
                true => WALL_TILE.glyph,
                false => '.',
            };
            term.put_char_formatted([p.x, p.y], c, fmt);
        }
    }

    if let Some(start) = finding.start {
        let fmt = CharFormat::new(Color::BLUE, Color::BLACK);
        term.put_char_formatted(start.into(), 'S', fmt);
    }

    if let Some(end) = finding.goal {
        let fmt = CharFormat::new(Color::BLUE, Color::BLACK);
        term.put_char_formatted(end.into(), 'E', fmt);
    }
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

fn read_inputs(
    mut evt: EventReader<InputCommand>,
    mut map: ResMut<SquareGrid>,
    mut path: ResMut<PathingState>,
) {
    for evt in evt.iter() {
        match evt {
            InputCommand::ToggleWall(pos) => {
                let loc = GridLocation { x: pos.x, y: pos.y };
                if map.passable(&loc) {
                    map.add_rect(loc.x, loc.y);
                } else {
                    map.rm_rect(loc.x, loc.y);
                }
                path.path = None;
                path.visited = None;
                path.time = Duration::ZERO;
            }
            InputCommand::SetPath(pos) => match (path.start, path.goal) {
                (Some(_), None) => {
                    path.goal = Some(*pos);
                }
                _ => {
                    path.clear();
                    path.start = Some(*pos);
                }
            },
        }
    }
}

fn input_to_commands(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    map: Res<SquareGrid>,
    mut input_writer: EventWriter<InputCommand>,
) {
    if let Some(window) = windows.get_primary() {
        if let Some(cur_pos) = window.cursor_position() {
            let (cam, t, proj) = q_cam.single();
            if let Some(pos) = proj.screen_to_world(cam, &windows, t, cur_pos) {
                let pos = world_to_map(&map, pos);
                let loc = GridLocation { x: pos.x, y: pos.y };
                if map.is_bounds(&loc) {
                    if input.just_pressed(MouseButton::Left) {
                        input_writer.send(InputCommand::ToggleWall(pos));
                    }

                    if input.just_pressed(MouseButton::Right) && map.passable(&loc) {
                        input_writer.send(InputCommand::SetPath(pos));
                    }
                }
            }
        }
    }
}

fn world_to_map(map: &SquareGrid, pos: Vec3) -> IVec2 {
    let pos = pos.truncate().floor().as_ivec2();
    let size = IVec2::new(map.width, map.height);
    pos + size / 2
}

fn update_path(map: Res<SquareGrid>, mut finding: ResMut<PathingState>) {
    if !map.is_changed() && !finding.is_changed() {
        return;
    }

    if let (Some(start), Some(goal)) = (finding.start, finding.goal) {
        let times = Instant::now();
        let start = GridLocation {
            x: start.x,
            y: start.y,
        };
        let goal = GridLocation {
            x: goal.x,
            y: goal.y,
        };
        let (path, visited) = a_star_search(&map, &start, &goal);
        if path.contains_key(&goal) {
            finding.path = Some(SquareGrid::reconstruct_path(&start, &goal, &path));
        }
        finding.visited = Some(visited);
        finding.time = times.elapsed();
    }
}

pub fn a_star_search(
    map: &SquareGrid,
    start: &GridLocation,
    goal: &GridLocation,
) -> (
    HashMap<GridLocation, GridLocation>,
    HashMap<GridLocation, i32>,
) {
    let mut frontier = BinaryHeap::new();
    frontier.push(Reverse(Pair(0, start.clone())));

    let mut came_from = HashMap::from([(start.clone(), start.clone())]);
    let mut cost_so_far = HashMap::from([(start.clone(), 0)]);

    while let Some(Reverse(Pair(_, current))) = frontier.pop() {
        if &current == goal {
            break;
        }

        for next in map.neighbors(&current) {
            let cost_current = cost_so_far.get(&current).unwrap_or(&0);
            let new_cost = cost_current + map.cost(&next);
            let prev_next_cost = cost_so_far.get(&next);
            if prev_next_cost.is_none() || &new_cost < prev_next_cost.unwrap_or(&0) {
                cost_so_far.insert(next, new_cost);
                came_from.insert(next, current);
                frontier.push(Reverse(Pair(
                    new_cost + SquareGrid::heuristic(&next, goal),
                    next,
                )));
            }
        }
    }
    (came_from, cost_so_far)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_plugin(TiledCameraPlugin)
        .add_startup_system(setup)
        .add_system(draw)
        .add_system(input_to_commands)
        .add_system(update_path)
        .add_system(read_inputs)
        .add_event::<InputCommand>()
        .run();
}
