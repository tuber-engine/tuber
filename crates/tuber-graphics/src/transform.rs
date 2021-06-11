use cgmath::{vec3, Deg, Matrix4};
use tuber_common::transform::Transform2D;

pub trait IntoMatrix4 {
    fn into_matrix4(self) -> cgmath::Matrix4<f32>;
}

impl IntoMatrix4 for Transform2D {
    fn into_matrix4(self) -> Matrix4<f32> {
        let translate_to_rotation_center =
            vec3(self.rotation_center.0, self.rotation_center.1, 0.0);

        cgmath::Matrix4::from_scale(self.scale)
            * cgmath::Matrix4::from_translation(vec3(self.translation.0, self.translation.1, 0.0))
            * cgmath::Matrix4::from_translation(translate_to_rotation_center.clone())
            * cgmath::Matrix4::from_angle_z(Deg(self.angle))
            * cgmath::Matrix4::from_translation(-translate_to_rotation_center)
    }
}
