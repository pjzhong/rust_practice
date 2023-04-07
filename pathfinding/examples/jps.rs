use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashSet, BTreeSet},
    rc::Rc, f32::consts::SQRT_2,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
enum GraphNode {
    Node(i32, i32),
    Wall(i32, i32),
}

impl GraphNode {
    fn get_x(&self) -> i32 {
        match self {
            GraphNode::Node(x, _) => *x,
            GraphNode::Wall(x, _) => *x,
        }
    }

    fn get_y(&self) -> i32 {
        match self {
            GraphNode::Node(_, y) => *y,
            GraphNode::Wall(_, y) => *y,
        }
    }
}

pub struct Graph {
    nodes: Vec<Rc<GraphNode>>,
    width: usize,
}

impl Graph {
    pub fn new(width: usize, height: usize) -> Self {
        let mut nodes = Vec::with_capacity(width * height);
        for w in 0..width {
            for h in 0..height {
                nodes.push(Rc::new(GraphNode::Node(w as i32, h as i32)));
            }
        }

        Self { nodes, width }
    }

    fn get_idx(&self, x: i32, y: i32) -> usize {
        let width = self.width as i32;
        (x + y * width) as usize
    }

    pub fn auto_build(width: usize, height: usize) -> Self {
        //噪音函数，自动生成阻挡物
        let fbm = Fbm::new()
            .set_octaves(16)
            .set_frequency(1.5)
            .set_lacunarity(3.0)
            .set_persistence(0.9);
        let plane = PlaneMapBuilder::new(&fbm).set_size(width, height).build();

        let mut graph = Self::new(width, height);
        //阻挡物生成阈值
        let threshold = 0.3;
        for w in 0..width {
            for h in 0..height {
                if threshold < plane.get_value(w, h) {
                    graph.set_wall(w as i32, h as i32)
                }
            }
        }

        graph
    }

    fn walkable_node(&self, x: i32, y: i32) -> Option<Rc<GraphNode>> {
        let idx = self.get_idx(x, y);
        match self.nodes.get(idx) {
            Some(rc) => match rc.as_ref() {
                node @ GraphNode::Node(..) => Some(rc.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.walkable_node(x, y).is_some()
    }

    pub fn is_not_walkable(&self, x: i32, y: i32) -> bool {
        !self.is_walkable(x, y)
    }

    pub fn set_wall(&mut self, x: i32, y: i32) {
        let idx = self.get_idx(x, y);

        if let Some(node) = self.nodes.get_mut(idx) {
            *node = Rc::new(GraphNode::Wall(x as i32, y as i32));
        }
    }

    pub fn set_node(&mut self, x: i32, y: i32) {
        let idx = self.get_idx(x, y);

        if let Some(node) = self.nodes.get_mut(idx) {
            *node = Rc::new(GraphNode::Node(x as i32, y as i32));
        }
    }
}

struct Heuristic;

impl Heuristic {
    pub fn octile(dx: f32, dy:f32) -> f32 {
        let squart2 = SQRT_2;
        if dx < dy {
            squart2 * dx + dy
        } else {
            squart2 * dy + dx
        }
    }

    pub fn manhattan(dx: f32, dy:f32) -> f32 {
        dx + dy
    }
}

struct Jps;

struct PathNode {
    node: Rc<GraphNode>,
    parent: Option<Rc<PathNode>>,
    // distance to start + estimate to end
    f: f32,
    // distance to start (parent's g + distance from parent)
    g: f32,
    // estimate to end
    h: f32,
}

impl PathNode {
    fn new(node: Rc<GraphNode>) -> Self {
        Self {
            node,
            parent: None,
            f: 0.0,
            g: 0.0,
            h: 0.0,
        }
    }

    fn get_x(&self) -> i32 {
        self.node.get_x()
    }

    fn get_y(&self) -> i32 {
        self.node.get_y()
    }

    fn set_parent(&mut self, parent: Rc<PathNode>) {
        self.parent = Some(parent);
    }
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f.total_cmp(&other.f)
    }
}

impl Jps {
    pub fn find_path(graph: &Graph, start: GraphNode, end: GraphNode) {
        if graph.is_not_walkable(end.get_x(), end.get_y()) {
            return;
        }

        let mut open: BTreeSet<PathNode> = BTreeSet::new();

        open.insert(PathNode::new(Rc::new(start)));

        let mut closed: HashSet<Rc<GraphNode>> = HashSet::new();
        while let Some(path_node) = open.pop_first() {
            closed.insert(path_node.node.clone());

            if path_node.node.as_ref() == &end {
                //TODO backtrace
                return;
            }

            Jps::identify_successors(graph, Rc::new(path_node), &end, &mut closed, &mut open)
        }
    }

    fn identify_successors(
        graph: &Graph,
        node: Rc<PathNode>,
        end: &GraphNode,
        closed: &mut HashSet<Rc<GraphNode>>,
        open: &mut BTreeSet<PathNode>,
    ) {
        let neighbors = Jps::find_neighbors(graph, node.clone());
        for neighbor in neighbors {
            let jump_point = Jps::jump(graph, neighbor.node.as_ref(), &node.node, end);
            if let Some(jump_point) = jump_point {
                if closed.contains(&jump_point) {
                    continue;
                }

                let dx = (jump_point.get_x() - node.get_x()).abs() as f32;
                let dy = (jump_point.get_y() - node.get_y()).abs() as f32;
                let d = Heuristic::octile(dx, dy);
                let ng = node.g + d;//next 'g' value

                open
            }
        }
    }

    fn jump(
        graph: &Graph,
        current: &GraphNode,
        neighbor: &GraphNode,
        goal: &GraphNode,
    ) -> Option<Rc<GraphNode>> {
        if graph.is_not_walkable(current.get_x(), current.get_y()) {
            return None;
        }

        if current == goal {
            return graph.walkable_node(current.get_x(), current.get_y());
        }

        let (x, y) = (current.get_x(), current.get_y());
        let (dx, dy) = (
            current.get_x() - neighbor.get_x(),
            current.get_y() - neighbor.get_y(),
        );

        // check for forced neighbors
        // along the diagonal
        if dx != 0 && dy != 0 {
            if (graph.is_walkable(x - dx, y + dy) && graph.is_not_walkable(x - dx, y))
                || (graph.is_walkable(x + dx, y - dy) && graph.is_not_walkable(x, y - dy))
            {
                return graph.walkable_node(x, y);
            }

            for next in vec![
                graph.walkable_node(x + dx, y),
                graph.walkable_node(x, y + dy),
            ]
            .into_iter()
            .flatten()
            {
                if Jps::jump(graph, &next, current, goal).is_some() {
                    return graph.walkable_node(x, y);
                }
            }
        } else {
            // check horizontally/vertically

            if dx != 0 {
                if graph.is_walkable(x + dx, y + 1) && graph.is_not_walkable(x, y + 1)
                    || graph.is_walkable(x + dx, y - 1) && graph.is_not_walkable(x, y - 1)
                {
                    return graph.walkable_node(x, y);
                }
            } else {
                if graph.is_walkable(x + 1, y + dy) && graph.is_not_walkable(x, y + 1)
                    || graph.is_walkable(x + -1, y + dy) && graph.is_not_walkable(x, y - 1)
                {
                    return graph.walkable_node(x, y);
                }
            }
        }

        // moving diagonally, must make sure one of the vertical/hhorizontal
        // neighbors is open to allow the path
        if graph.is_walkable(x + dx, y) || graph.is_walkable(x, y + dy) {
            if let Some(next) = graph.walkable_node(x + dx, y + dy) {
                Jps::jump(graph, &next, current, goal)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// https://zerowidth.com/2013/a-visual-explanation-of-jump-point-search.html
    fn find_neighbors(graph: &Graph, parent: Rc<PathNode>) -> Vec<PathNode> {
        if let Some(parent) = parent.parent.as_ref() {
            let (x, y) = (parent.get_x(), parent.get_y());

            let (dx, dy) = (
                (x - parent.get_x()) / 1.max((x - parent.get_x()).abs()),
                (y - parent.get_y()) / 1.max((x - parent.get_y()).abs()),
            );

            if dx != 0 && dy != 0 {
                let mut vec = vec![];
                let horizonetal = graph.walkable_node(x, y + dy);
                let vertically = graph.walkable_node(x + dx, y);

                // moving horizonetally and vertically first
                if let Some(node) = horizonetal.as_ref() {
                    vec.push(node.clone());
                };
                if let Some(node) = vertically.as_ref() {
                    vec.push(node.clone());
                };

                // moving  diagonally
                if horizonetal.is_some() || vertically.is_some() {
                    if let Some(node) = graph.walkable_node(x + dx, y + dy) {
                        vec.push(node)
                    }
                }

                // force neighbors
                if graph.is_not_walkable(x - dx, y) && graph.is_walkable(x, y + dy) {
                    if let Some(node) = graph.walkable_node(x - dx, y + dy) {
                        vec.push(node);
                    }
                }

                if graph.is_not_walkable(x, y - dy) && graph.is_walkable(x + dx, y) {
                    if let Some(node) = graph.walkable_node(x + dx, y - dy) {
                        vec.push(node);
                    }
                }

                to_path_node(vec, parent)
            } else {
                // search horizonetally
                if dx == 0 {
                    let mut vec = vec![];
                    if let Some(node) = graph.walkable_node(x, y + dy) {
                        vec.push(node);

                        //down is force neighbors
                        if graph.is_not_walkable(x + 1, y) {
                            if let Some(node) = graph.walkable_node(x + 1, y + dy) {
                                vec.push(node);
                            }
                        }

                        // up is force neighbors
                        if graph.is_not_walkable(x - 1, y) {
                            if let Some(node) = graph.walkable_node(x - 1, y + dy) {
                                vec.push(node);
                            }
                        }
                    }

                    to_path_node(vec, parent)
                }
                // search vertically
                else if let Some(node) = graph.walkable_node(x + dx, y) {
                    let mut vec = vec![];
                    vec.push(node);

                    // right is force neighbors
                    if graph.is_not_walkable(x, y + 1) {
                        if let Some(node) = graph.walkable_node(x + dx, y + 1) {
                            vec.push(node);
                        }
                    }

                    //left is force neighbors
                    if graph.is_not_walkable(x, y - 1) {
                        if let Some(node) = graph.walkable_node(x + dx, y - 1) {
                            vec.push(node);
                        }
                    }

                    to_path_node(vec, parent)
                } else {
                    vec![]
                }
            }

            // no neighbors
        } else {
            let vec = Jps::get_all_neightbors(graph, &parent.node);
            to_path_node(vec, &parent)
        }
    }

    fn get_all_neightbors(graph: &Graph, node: &GraphNode) -> Vec<Rc<GraphNode>> {
        let (x, y) = (node.get_x(), node.get_y());

        let n = graph.walkable_node(x, y - 1);
        let e = graph.walkable_node(x + 1, y);
        let s = graph.walkable_node(x, y + 1);
        let w = graph.walkable_node(x - 1, y);

        let nw = if n.is_some() || w.is_some() {
            graph.walkable_node(x - 1, x - 1)
        } else {
            None
        };
        let ne = if n.is_some() || e.is_some() {
            graph.walkable_node(x + 1, y - 1)
        } else {
            None
        };
        let se = if s.is_some() || e.is_some() {
            graph.walkable_node(x + 1, y + 1)
        } else {
            None
        };
        let sw = if s.is_some() || w.is_some() {
            graph.walkable_node(x - 1, y + 1)
        } else {
            None
        };

        vec![n, e, s, w, nw, ne, se, sw]
            .into_iter()
            .flatten()
            .collect()
    }
}

fn to_path_node(vec: Vec<Rc<GraphNode>>, parent: &Rc<PathNode>) -> Vec<PathNode> {
    let mut res = vec![];
    for graph_node in vec {
        let mut path_node = PathNode::new(graph_node);
        path_node.set_parent(parent.clone());

        res.push(path_node);
    }
    res
}

fn main() {
    let mut graph = Graph::auto_build(255, 255);
    graph.set_node(33, 33);
    graph.set_node(99, 99);

    Jps::find_path(&graph, GraphNode::Node(33, 33), GraphNode::Node(99, 99))
}
