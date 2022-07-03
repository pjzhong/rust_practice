use crate::hittable::{HitRecord, Hittable};
use crate::{Axis, Ray, AABB};
use std::cmp::Ordering;
use std::sync::Arc;

pub struct BvhNode {
    contents: BvhContent,
    aabb: Option<AABB>,
}

pub enum BvhContent {
    Node {
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
    Leaf(Arc<dyn Hittable>),
}
impl BvhNode {
    pub fn new(objects: &mut [Arc<dyn Hittable>], time0: f32, time1: f32) -> Self {
        let axis = {
            let mut ranges = [
                (Axis::X, BvhNode::axis_range(objects, time0, time1, Axis::X)),
                (Axis::Y, BvhNode::axis_range(objects, time0, time1, Axis::Y)),
                (Axis::Z, BvhNode::axis_range(objects, time0, time1, Axis::Z)),
            ];

            ranges.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            ranges[0].0
        };

        objects.sort_unstable_by(|a, b| {
            match (a.bounding_box(time0, time1), b.bounding_box(time0, time1)) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => {
                    let av = a.minimum[axis] + a.maximum[axis];
                    let bv = b.minimum[axis] + b.maximum[axis];
                    av.partial_cmp(&bv).unwrap()
                }
            }
        });

        match objects.len() {
            0 => panic!("Can't create a BCH from zero objects."),
            1 => BvhNode {
                aabb: objects[0].bounding_box(time0, time1),
                contents: BvhContent::Leaf(objects[0].clone()),
            },
            _ => {
                let mid = objects.len() / 2;
                let right = Box::new(BvhNode::new(&mut objects[0..mid], time0, time1));
                let left = Box::new(BvhNode::new(&mut objects[mid..], time0, time1));

                BvhNode {
                    aabb: match (left.aabb, right.aabb) {
                        (None, None) => None,
                        (a, None) | (None, a) => a,
                        (Some(a), Some(b)) => Some(AABB::surrounding_box(&a, &b)),
                    },
                    contents: BvhContent::Node { left, right },
                }
            }
        }
    }

    fn axis_range(objs: &[Arc<dyn Hittable>], time0: f32, time1: f32, axis: Axis) -> f32 {
        let rang = objs.iter().fold(f32::MAX..f32::MIN, |rang, o| {
            if let Some(bb) = o.bounding_box(time0, time1) {
                let min = bb.minimum[axis].min(bb.maximum[axis]);
                let max = bb.minimum[axis].max(bb.maximum[axis]);
                rang.start.min(min)..rang.end.max(max)
            } else {
                rang
            }
        });

        rang.end - rang.start
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if let Some(aabb) = self.aabb {
            if aabb.hit(r, t_min, t_max) {
                match &self.contents {
                    BvhContent::Node { left, right } => {
                        let hit_left = left.hit(r, t_min, t_max);

                        let t_max = if let Some(h) = &hit_left { h.t } else { t_max };

                        let hit_right = right.hit(r, t_min, t_max);

                        match (hit_left, hit_right) {
                            (h, None) | (None, h) => h,
                            (Some(l), Some(r)) => {
                                if l.t < r.t {
                                    Some(l)
                                } else {
                                    Some(r)
                                }
                            }
                        }
                    }
                    BvhContent::Leaf(obj) => obj.hit(r, t_min, t_max),
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        self.aabb
    }
}
