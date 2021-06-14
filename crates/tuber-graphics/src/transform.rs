use nalgebra::{Matrix, Matrix3, Matrix4, Transform2, Vector3};
use tuber_common::transform::Transform2D;

pub trait IntoMatrix4 {
    fn into_matrix4(self) -> Matrix4<f32>;
}

impl IntoMatrix4 for Transform2D {
    fn into_matrix4(self) -> Matrix4<f32> {
        let translate_to_rotation_center =
            Vector3::new(self.rotation_center.0, self.rotation_center.1, 0.0);

        Matrix4::new_scaling(self.scale)
            * Matrix4::new_translation(&Vector3::new(self.translation.0, self.translation.1, 0.0))
            * Matrix4::new_translation(&translate_to_rotation_center.clone())
            * Matrix4::new_rotation(Vector3::new(0.0, 0.0, self.angle.to_radians()))
            * Matrix4::new_translation(&-&translate_to_rotation_center)
    }
}
