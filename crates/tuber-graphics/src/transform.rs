use cgmath::{vec3, Deg};

#[derive(Debug, Copy, Clone)]
pub struct Transform2D {
    pub translation: (f32, f32),
    pub angle: f32,
    pub rotation_center: (f32, f32),
    pub scale: f32,
}

impl From<Transform2D> for cgmath::Matrix4<f32> {
    fn from(transform_2d: Transform2D) -> Self {
        let translate_to_rotation_center = vec3(
            transform_2d.rotation_center.0,
            transform_2d.rotation_center.1,
            0.0,
        );

        cgmath::Matrix4::from_scale(transform_2d.scale)
            * cgmath::Matrix4::from_translation(vec3(
                transform_2d.translation.0,
                transform_2d.translation.1,
                0.0,
            ))
            * cgmath::Matrix4::from_translation(translate_to_rotation_center.clone())
            * cgmath::Matrix4::from_angle_z(Deg(transform_2d.angle))
            * cgmath::Matrix4::from_translation(-translate_to_rotation_center)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0),
            angle: 0.0,
            rotation_center: (0.0, 0.0),
            scale: 1.0,
        }
    }
}
