use crate::{CollisionShape, Vector2};

pub fn are_colliding(
    first_shape: &CollisionShape,
    second_shape: &CollisionShape,
) -> Option<CollisionData> {
    let mut shapes_overlap = f32::MAX;
    let mut smallest_axis = None;
    let mut axes = first_shape.polygon.axes();
    axes.append(&mut second_shape.polygon.axes());

    for axis in &axes {
        let projected_first_shape = first_shape.polygon.project(axis);
        let projected_second_shape = second_shape.polygon.project(axis);

        if !projections_overlap(projected_first_shape, projected_second_shape) {
            return None;
        } else {
            let o = overlap(projected_first_shape, projected_second_shape);
            if o < shapes_overlap {
                shapes_overlap = o;
                smallest_axis = Some(axis.clone());
            }
        }
    }

    Some(CollisionData::new(smallest_axis.unwrap(), shapes_overlap))
}

#[derive(Clone)]
pub struct CollisionData {
    minimum_translation_vector: Vector2,
}

impl CollisionData {
    pub fn new(smallest_axis: Vector2, magnitude: f32) -> Self {
        let initial_magnitude =
            (smallest_axis.x * smallest_axis.x + smallest_axis.y * smallest_axis.y).sqrt();
        Self {
            minimum_translation_vector: Vector2::new(
                smallest_axis.x * (magnitude / initial_magnitude),
                smallest_axis.y * (magnitude / initial_magnitude),
            ),
        }
    }

    pub fn minimum_translation_vector(&self) -> &Vector2 {
        &self.minimum_translation_vector
    }
}

fn overlap(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    p1.1.min(p2.1) - p1.0.max(p2.0)
}

fn projections_overlap(p1: (f32, f32), p2: (f32, f32)) -> bool {
    p1.0 < p2.1 && p2.0 <= p1.1
}
