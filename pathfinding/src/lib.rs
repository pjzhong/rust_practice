use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, LinkedList},
    hash::Hash,
};

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Debug)]
pub struct GridLocation {
    pub x: i32,
    pub y: i32,
}

const GRID_DIR: [GridLocation; 8] = [
    GridLocation { x: 1, y: 0 },
    GridLocation { x: -1, y: 0 },
    GridLocation { x: 0, y: -1 },
    GridLocation { x: 0, y: 1 },
    GridLocation { x: 1, y: 1 },
    GridLocation { x: 1, y: -1 },
    GridLocation { x: -1, y: 1 },
    GridLocation { x: -1, y: -1 },
];

pub struct SquareGrid {
    pub width: i32,
    pub height: i32,
    pub walls: HashSet<GridLocation>,
    pub forest: HashSet<GridLocation>,
}

impl SquareGrid {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            walls: HashSet::new(),
            forest: HashSet::new(),
        }
    }

    pub fn passable(&self, id: &GridLocation) -> bool {
        self.walls.get(id).is_none()
    }

    pub fn cost(&self, to_node: &GridLocation) -> i32 {
        1
    }

    pub fn is_bounds(&self, id: &GridLocation) -> bool {
        0 <= id.x && id.x < self.width && 0 <= id.y && id.y < self.height
    }

    pub fn neighbors(&self, id: &GridLocation) -> Vec<GridLocation> {
        let mut result = vec![];

        for dir in GRID_DIR {
            let next = GridLocation {
                x: id.x + dir.x,
                y: id.y + dir.y,
            };

            if self.is_bounds(&next) && self.passable(&next) {
                result.push(next);
            }
        }
        result
    }

    pub fn add_rects(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        for x in x1..x2 {
            for y in y1..y2 {
                self.walls.insert(GridLocation { x, y });
            }
        }
    }

    pub fn add_rect(&mut self, x: i32, y: i32) {
        self.walls.insert(GridLocation { x, y });
    }

    pub fn rm_rect(&mut self, x: i32, y: i32) {
        self.walls.remove(&GridLocation { x, y });
    }

    pub fn add_forst(&mut self, x: i32, y: i32) {
        self.forest.insert(GridLocation { x, y });
    }

    pub fn make_diagram1() -> SquareGrid {
        let mut grid = SquareGrid::new(30, 15);
        grid.add_rects(3, 3, 5, 12);
        grid.add_rects(13, 4, 15, 15);
        grid.add_rects(21, 0, 23, 7);
        grid.add_rects(23, 5, 26, 7);

        grid
    }

    pub fn make_diagram2() -> SquareGrid {
        let mut grid = SquareGrid::new(10, 10);
        grid.add_rects(1, 7, 4, 9);

        grid.add_forst(3, 4);
        grid.add_forst(3, 5);
        grid.add_forst(4, 1);
        grid.add_forst(4, 2);
        grid.add_forst(4, 3);
        grid.add_forst(4, 4);
        grid.add_forst(4, 5);
        grid.add_forst(4, 6);
        grid.add_forst(4, 7);
        grid.add_forst(4, 8);
        grid.add_forst(5, 1);
        grid.add_forst(5, 2);
        grid.add_forst(5, 3);
        grid.add_forst(5, 4);
        grid.add_forst(5, 5);
        grid.add_forst(5, 6);
        grid.add_forst(5, 7);
        grid.add_forst(5, 8);
        grid.add_forst(6, 2);
        grid.add_forst(6, 3);
        grid.add_forst(6, 4);
        grid.add_forst(6, 5);
        grid.add_forst(6, 6);
        grid.add_forst(6, 7);
        grid.add_forst(7, 3);
        grid.add_forst(7, 4);
        grid.add_forst(7, 5);

        grid
    }

    pub fn make_diagram3() -> SquareGrid {
        let mut grid = SquareGrid::new(30, 30);
        grid.add_rects(3, 3, 5, 12);
        grid.add_rects(13, 4, 15, 15);
        grid.add_rects(21, 0, 23, 7);
        grid.add_rects(23, 5, 26, 7);

        grid.add_rects(25, 10, 29, 20);
        grid.add_rects(15, 3, 20, 8);
        grid
    }

    fn breath_first_search(
        &self,
        start: &GridLocation,
        goal: &GridLocation,
    ) -> HashMap<GridLocation, GridLocation> {
        let mut frontier = LinkedList::new();
        frontier.push_back(start.clone());

        let mut came_from = HashMap::new();
        came_from.insert(start.clone(), start.clone());

        while let Some(current) = frontier.pop_front() {
            if &current == goal {
                break;
            }

            for next in self.neighbors(&current) {
                if let None = came_from.get(&next) {
                    frontier.push_back(next);
                    came_from.insert(next, current);
                }
            }
        }

        came_from
    }

    fn dijkstra_search(
        &self,
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

        while let Some(Reverse(Pair(current_cost, current))) = frontier.pop() {
            if &current == goal {
                break;
            }

            for next in self.neighbors(&current) {
                let new_cost = current_cost + self.cost(&next);
                let prev_next_cost = cost_so_far.get(&next);
                if prev_next_cost.is_none() || &new_cost < prev_next_cost.unwrap_or(&0) {
                    cost_so_far.insert(next, new_cost);
                    came_from.insert(next, current);
                    frontier.push(Reverse(Pair(new_cost, next)));
                }
            }
        }

        (came_from, cost_so_far)
    }

    pub fn reconstruct_path(
        start: &GridLocation,
        goal: &GridLocation,
        cam_from: &HashMap<GridLocation, GridLocation>,
    ) -> Vec<GridLocation> {
        let mut path = vec![];
        let mut current = goal;
        while current != start {
            path.push(current.clone());
            current = cam_from
                .get(&current)
                .unwrap_or_else(|| &GridLocation { x: -1, y: -1 });
        }

        path.push(start.clone());
        path.reverse();
        path
    }

    pub fn a_star_search(
        &self,
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

            for next in self.neighbors(&current) {
                let cost_current = cost_so_far.get(&current).unwrap_or(&0);
                let new_cost = cost_current + self.cost(&next);
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

    pub fn heuristic(a: &GridLocation, b: &GridLocation) -> i32 {
        let res = ((a.x - b.x).abs().pow(2) + (a.y - b.y).abs().pow(2)) as f64;
        res.sqrt() as i32
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Pair<A, B>(pub A, pub B);

fn draw_grid(
    graph: &SquareGrid,
    point_to: Option<HashMap<GridLocation, GridLocation>>,
    distance: Option<HashMap<GridLocation, i32>>,
    path: Option<Vec<GridLocation>>,
    start: Option<GridLocation>,
    goal: Option<GridLocation>,
) {
    let point_to = point_to.unwrap_or_else(|| HashMap::new());
    let distance = distance.unwrap_or_else(|| HashMap::new());

    let start = start.unwrap_or_else(|| GridLocation { x: -1, y: -1 });
    let goal = goal.unwrap_or_else(|| GridLocation { x: -1, y: -1 });
    let path = path.unwrap_or_else(|| vec![]);
    for y in 0..graph.height {
        for x in 0..graph.width {
            let id = GridLocation { x, y };

            if graph.walls.contains(&id) {
                print!("###");
            } else if id == start {
                print!(" A ");
            } else if id == goal {
                print!(" Z ");
            } else if path.contains(&id) {
                print!("{:3}", '@');
            } else if let Some(d) = distance.get(&id) {
                print!("{:3}", d);
            } else if let Some(next) = point_to.get(&id) {
                if next.x == x + 1 {
                    print!(" > ");
                } else if next.x == x - 1 {
                    print!(" < ");
                } else if next.y == y + 1 {
                    print!(" v ");
                } else if next.y == y - 1 {
                    print!(" ^ ");
                } else {
                    print!("{:3}", '.');
                }
            } else {
                print!("{:3}", '.');
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_diagram1() {
        let grid = SquareGrid::make_diagram1();
        draw_grid(&grid, None, None, None, None, None);
    }

    #[test]
    fn draw_diagram2() {
        let grid = SquareGrid::make_diagram2();
        let start = GridLocation { x: 7, y: 8 };
        let goal = GridLocation { x: 17, y: 2 };
        let parents = grid.breath_first_search(&start, &goal);
        draw_grid(&grid, Some(parents), None, None, Some(start), Some(goal));
    }

    #[test]
    fn draw_diagram3() {
        let grid = SquareGrid::make_diagram2();
        let start = GridLocation { x: 1, y: 4 };
        let goal = GridLocation { x: 8, y: 3 };
        let (parents, cost) = grid.dijkstra_search(&start, &goal);
        let paths = SquareGrid::reconstruct_path(&start, &goal, &parents);
        draw_grid(&grid, Some(parents), None, None, Some(start), Some(goal));
        println!();
        draw_grid(&grid, None, None, Some(paths), Some(start), Some(goal));
        println!();
        draw_grid(&grid, None, Some(cost), None, Some(start), Some(goal));
    }

    #[test]
    fn draw_diagram4() {
        let grid = SquareGrid::make_diagram3();
        let start = GridLocation { x: 1, y: 4 };
        let goal = GridLocation { x: 29, y: 29 };
        let (parents, cost) = grid.a_star_search(&start, &goal);
        let paths = SquareGrid::reconstruct_path(&start, &goal, &parents);
        draw_grid(&grid, Some(parents), None, None, Some(start), Some(goal));
        println!();
        draw_grid(&grid, None, None, Some(paths), Some(start), Some(goal));
        println!();
        draw_grid(&grid, None, Some(cost), None, Some(start), Some(goal));
    }
}
