use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashSet},
    rc::Rc,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
enum GrapNode {
    Node(i32, i32),
    Wall(i32, i32),
}

impl GrapNode {
    fn get_x(&self) -> i32 {
        match self {
            GrapNode::Node(x, _) => *x,
            GrapNode::Wall(x, _) => *x,
        }
    }

    fn get_y(&self) -> i32 {
        match self {
            GrapNode::Node(_, y) => *y,
            GrapNode::Wall(_, y) => *y,
        }
    }
}

pub struct Graph {
    nodes: Vec<Rc<GrapNode>>,
    width: usize,
}

impl Graph {
    pub fn new(width: usize, height: usize) -> Self {
        let mut nodes = Vec::with_capacity(width * height);
        for w in 0..width {
            for h in 0..height {
                nodes.push(Rc::new(GrapNode::Node(w as i32, h as i32)));
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

    fn walkable_node(&self, x: i32, y: i32) -> Option<Rc<GrapNode>> {
        let idx = self.get_idx(x, y);
        match self.nodes.get(idx) {
            Some(rc) => match rc.as_ref() {
                node @ GrapNode::Node(..) => Some(rc.clone()),
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
            *node = Rc::new(GrapNode::Wall(x as i32, y as i32));
        }
    }

    pub fn set_node(&mut self, x: i32, y: i32) {
        let idx = self.get_idx(x, y);

        if let Some(node) = self.nodes.get_mut(idx) {
            *node = Rc::new(GrapNode::Node(x as i32, y as i32));
        }
    }
}

struct Jps;

struct PathNode {
    node: Rc<GrapNode>,
    parent: Option<Rc<PathNode>>,
    // distance to start + estimate to end
    f: f64,
    // distance to start (parent's g + distance from parent)
    g: f64,
    // estimate to end
    h: f64,
}

impl PathNode {
    fn new(node: GrapNode) -> Self {
        Self {
            node: Rc::new(node),
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
    pub fn find_path(grpah: &Graph, start: GrapNode, end: GrapNode) {
        if grpah.is_not_walkable(end.get_x(), end.get_y()) {
            return;
        }

        let mut open: BinaryHeap<Reverse<PathNode>> = BinaryHeap::new();

        open.push(Reverse(PathNode::new(start)));

        let mut closed: HashSet<Rc<GrapNode>> = HashSet::new();
        while let Some(Reverse(path_node)) = open.pop() {
            closed.insert(path_node.node.clone());

            if path_node.node.as_ref() == &end {
                return;
            }
        }
    }

    fn identify_successors(
        grpah: &Graph,
        path_node: Rc<PathNode>,
        end: &GrapNode,
        closed: &mut HashSet<GrapNode>,
        open: &mut BinaryHeap<Reverse<PathNode>>,
    ) {
    }

    fn jump(graph: &Graph, current: &GrapNode, neighbor: &GrapNode) {
        if graph.is_not_walkable(neighbor.get_x(), neighbor.get_y()) {
            return;
        }
    }

    /// https://zerowidth.com/2013/a-visual-explanation-of-jump-point-search.html
    fn find_neighbors(graph: &Graph, node: &PathNode) -> Vec<Rc<GrapNode>> {
        if let Some(parent) = node.parent.as_ref() {
            let (x, y) = (node.get_x(), node.get_y());

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
            } else {
                // search horizonetally
                if dx == 0 {
                    if let Some(node) = graph.walkable_node(x, y + dy) {
                        let mut vec = vec![];
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

                        return vec;
                    }
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

                    return vec;
                }
            }

            // no neighbors
            vec![]
        } else {
            Jps::get_all_neightbors(graph, &node.node)
        }
    }

    fn get_all_neightbors(graph: &Graph, node: &GrapNode) -> Vec<Rc<GrapNode>> {
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

fn main() {
    let mut graph = Graph::auto_build(255, 255);
    graph.set_node(33, 033);
    graph.set_node(99, 99);

    Jps::find_path(&graph, GrapNode::Node(33, 33), GrapNode::Node(99, 99))
}
