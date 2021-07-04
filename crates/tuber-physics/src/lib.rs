mod sat;

use nalgebra::{Point2, Point3};
use std::collections::{HashMap, HashSet};
use tuber_common::transform::{IntoMatrix4, Transform2D};
use tuber_core::DeltaTime;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::{R, W};
use tuber_ecs::system::SystemBundle;

type Vector2 = nalgebra::Vector2<f32>;

pub struct Physics {
    gravity: Vector2,
}

impl Physics {
    pub fn new(gravity: (f32, f32)) -> Self {
        Self {
            gravity: Vector2::new(gravity.0, gravity.1),
        }
    }

    pub fn update_rigid_body_2d(
        &mut self,
        delta_time: f64,
        transform: &mut Transform2D,
        rigid_body: &mut RigidBody2D,
    ) {
        if !rigid_body.grounded {
            rigid_body.acceleration += self.gravity;
        }
        rigid_body.velocity = rigid_body.velocity + rigid_body.acceleration * delta_time as f32;
        transform.translation.0 += rigid_body.velocity.x;
        transform.translation.1 += rigid_body.velocity.y;
    }

    pub fn default_system_bundle() -> SystemBundle {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(physics_update_system);
        system_bundle
    }
}

pub fn physics_update_system(ecs: &mut Ecs) {
    let DeltaTime(delta_time) = *ecs
        .shared_resource::<DeltaTime>()
        .expect("DeltaTime resource not found");
    let mut physics = ecs
        .shared_resource_mut::<Physics>()
        .expect("No Physics resource");

    for (_, (mut transform, mut rigid_body)) in ecs.query::<(W<Transform2D>, W<RigidBody2D>)>() {
        physics.update_rigid_body_2d(delta_time, &mut transform, &mut rigid_body);
    }

    let mut displacements = HashMap::new();
    let mut collided = HashSet::new();

    for (first, (transform, collision_shapes)) in
        ecs.query::<(R<Transform2D>, R<CollisionShapes>)>()
    {
        for (second, (second_transform, second_collision_shapes)) in
            ecs.query::<(R<Transform2D>, R<CollisionShapes>)>()
        {
            if first == second {
                continue;
            }

            for collision_shape in &collision_shapes.shapes {
                for second_collision_shape in &second_collision_shapes.shapes {
                    let transformed_collision_box = collision_shape.transform(&transform);
                    let transformed_second_collision_box =
                        second_collision_shape.transform(&second_transform);

                    if let Some(collision_data) = sat::are_colliding(
                        &transformed_collision_box,
                        &transformed_second_collision_box,
                    ) {
                        let displacement = Vector2::new(
                            -collision_data.smallest_axis.x,
                            collision_data.smallest_axis.y,
                        );

                        let s = (displacement.x * displacement.x + displacement.y * displacement.y)
                            .sqrt();

                        let displacement = (
                            displacement.x * collision_data.overlap / s,
                            displacement.y * collision_data.overlap / s,
                        );

                        displacements.insert(first, displacement);
                        collided.insert(first);
                    }
                }
            }
        }
    }

    for (id, (mut rigid_body,)) in ecs.query::<(W<RigidBody2D>,)>() {
        if !collided.contains(&id) {
            rigid_body.grounded = false;
        }
    }

    for id in collided {
        let displacement = displacements[&id];
        if let Some((_, (mut transform, mut body))) =
            ecs.query_one_by_id::<(W<Transform2D>, W<RigidBody2D>)>(id)
        {
            transform.translation.0 += displacement.0;
            transform.translation.1 += displacement.1;

            if displacement.1 < 0.0 {
                body.grounded = true;
                body.velocity.y = 0.0;
                body.acceleration.y = 0.0;
            }
            if displacement.1 > 0.0 {
                body.velocity.y = 0.0;
                body.acceleration.y = 0.0;
            }

            if (displacement.0 < 0.0) || (displacement.0 > 0.0) {
                body.velocity.x = 0.0;
                body.acceleration.x = 0.0;
            }
        }
    }
}

#[derive(Debug)]
pub struct RigidBody2D {
    pub velocity: Vector2,
    pub acceleration: Vector2,
    pub grounded: bool,
}

impl Default for RigidBody2D {
    fn default() -> Self {
        Self {
            velocity: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
            grounded: false,
        }
    }
}

pub struct StaticBody2D;

#[derive(Debug)]
pub struct Polygon {
    points: Vec<Point2<f32>>,
}

impl Polygon {
    pub fn axes(&self) -> Vec<Vector2> {
        let mut axes = vec![];
        let mut point_iterator = self.points.iter();
        let initial_point = point_iterator.next().unwrap();
        let mut first_point = initial_point;
        while let Some(next) = point_iterator.next() {
            let second_point = next;

            axes.push(
                Vector2::new(
                    second_point.y - first_point.y,
                    -(second_point.x - first_point.x),
                )
                .normalize(),
            );

            first_point = second_point;
        }
        axes
    }

    pub fn transform(&self, transform: &Transform2D) -> Self {
        let transform_matrix = transform.into_matrix4();
        Self {
            points: self
                .points
                .iter()
                .map(|point| {
                    (transform_matrix.transform_point(&Point3::new(point.x, point.y, 0.0))).xy()
                })
                .collect(),
        }
    }

    pub fn project(&self, axis: &Vector2) -> (f32, f32) {
        self.points[1..].iter().fold(
            (
                axis.dot(&self.points[0].coords),
                axis.dot(&self.points[0].coords),
            ),
            |(minimum, maximum), point| {
                let projected = axis.dot(&point.coords);
                if projected < minimum {
                    (projected, maximum)
                } else if projected > maximum {
                    (minimum, projected)
                } else {
                    (minimum, maximum)
                }
            },
        )
    }
}

#[derive(Debug)]
pub struct CollisionShapes {
    pub shapes: Vec<CollisionShape>,
}

#[derive(Debug)]
pub struct CollisionShape {
    polygon: Polygon,
}

impl CollisionShape {
    pub fn from_rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            polygon: Polygon {
                points: vec![
                    Point2::new(x, y),
                    Point2::new(x + width, y),
                    Point2::new(x + width, y + height),
                    Point2::new(x, y + height),
                ],
            },
        }
    }

    pub fn from_centered_rectangle(x_center: f32, y_center: f32, width: f32, height: f32) -> Self {
        Self {
            polygon: Polygon {
                points: vec![
                    Point2::new(x_center - width / 2.0, y_center - height / 2.0),
                    Point2::new(x_center + width / 2.0, y_center - height / 2.0),
                    Point2::new(x_center + width / 2.0, y_center + height / 2.0),
                    Point2::new(x_center - width / 2.0, y_center + height / 2.0),
                ],
            },
        }
    }

    pub fn transform(&self, transform: &Transform2D) -> Self {
        Self {
            polygon: self.polygon.transform(transform),
        }
    }
}
