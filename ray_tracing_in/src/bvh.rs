use crate::hittable::{HitRecord, Hittable};
use crate::{Ray, AABB};
use rand::Rng;
use std::cmp::Ordering;
use std::sync::Arc;

pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    aabb: AABB,
}

impl BvhNode {
    pub fn new(
        src_objects: &mut [Arc<dyn Hittable>],
        start: usize,
        end: usize,
        time0: f32,
        time1: f32,
    ) -> Option<Self> {
        let mut rang = rand::thread_rng();
        let comparator = match rang.gen_range(0..2) {
            0 => BvhNode::box_x_compare,
            1 => BvhNode::box_y_compare,
            _ => BvhNode::box_z_compare,
        };
        let object_span = end - start;

        let (left, right) = if object_span == 1 {
            (src_objects[start].clone(), src_objects[start].clone())
        } else if object_span == 2 {
            if Ordering::Less == comparator(&src_objects[start], &src_objects[start + 1]) {
                (src_objects[start].clone(), src_objects[start + 1].clone())
            } else {
                (src_objects[start + 1].clone(), src_objects[start].clone())
            }
        } else {
            src_objects[start..end].sort_by(comparator);

            let mid = start + object_span / 2;
            let left = BvhNode::new(src_objects, start, mid, time0, time1);
            let right = BvhNode::new(src_objects, mid, end, time0, time1);

            if let (Some(left), Some(right)) = (left, right) {
                (
                    Arc::new(left) as Arc<dyn Hittable>,
                    Arc::new(right) as Arc<dyn Hittable>,
                )
            } else {
                return None;
            }
        };

        let (box_left, box_right) = (
            left.bounding_box(time0, time1),
            right.bounding_box(time0, time1),
        );
        if let (Some(box_left), Some(box_right)) = (box_left, box_right) {
            Some(Self {
                left,
                right,
                aabb: AABB::surrounding_box(&box_left, &box_right),
            })
        } else {
            None
        }
    }

    fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis: usize) -> Option<Ordering> {
        let (box_a, box_b) = (a.bounding_box(0.0, 1.0), b.bounding_box(0.0, 1.0));

        if let (Some(box_a), Some(box_b)) = (box_a, box_b) {
            box_a.min()[axis].partial_cmp(&box_b.min()[axis])
        } else if box_a.is_some() {
            Some(Ordering::Less)
        } else {
            None
        }
    }

    fn box_x_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
        if let Some(b) = BvhNode::box_compare(a, b, 0) {
            b
        } else {
            Ordering::Greater
        }
    }

    fn box_y_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
        if let Some(b) = BvhNode::box_compare(a, b, 1) {
            b
        } else {
            Ordering::Greater
        }
    }

    fn box_z_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> Ordering {
        if let Some(b) = BvhNode::box_compare(a, b, 2) {
            b
        } else {
            Ordering::Greater
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if self.aabb.hit(r, t_min, t_max) {
            let hit_left = self.left.hit(r, t_min, t_max);
            let hit_right = self.right.hit(r, t_min, t_max);

            if hit_left.is_some() {
                hit_left
            } else if hit_right.is_some() {
                hit_right
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(self.aabb)
    }
}
