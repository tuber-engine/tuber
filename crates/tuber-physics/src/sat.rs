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
            if o.abs() < shapes_overlap {
                shapes_overlap = o.abs();

                let axis = Vector2::new(
                    if o < 0.0 { -axis.x } else { axis.x },
                    if o < 0.0 { axis.y } else { -axis.y },
                );
                smallest_axis = Some(axis);
            }
        }
    }

    Some(CollisionData {
        overlap: shapes_overlap,
        smallest_axis: smallest_axis.unwrap(),
    })
}

#[derive(Clone, Debug)]
pub struct CollisionData {
    pub overlap: f32,
    pub smallest_axis: Vector2,
}

fn overlap(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    if p1.1 < p2.1 {
        if p1.0 > p2.0 {
            p1.1 - p1.0
        } else {
            p1.1 - p2.0
        }
    } else {
        if p1.0 < p2.0 {
            p2.1 - p2.0
        } else {
            p1.0 - p2.1
        }
    }
}

fn projections_overlap(p1: (f32, f32), p2: (f32, f32)) -> bool {
    p1.0 < p2.1 && p2.0 <= p1.1
}
