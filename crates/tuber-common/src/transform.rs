use nalgebra::{Matrix4, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Transform2D {
    pub translation: (f32, f32),
    pub angle: f32,
    pub rotation_center: (f32, f32),
    pub scale: (f32, f32),
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0),
            angle: 0.0,
            rotation_center: (0.0, 0.0),
            scale: (1.0, 1.0),
        }
    }
}

pub trait IntoMatrix4 {
    fn into_matrix4(self) -> Matrix4<f32>;
}

impl IntoMatrix4 for Transform2D {
    fn into_matrix4(self) -> Matrix4<f32> {
        let translate_to_rotation_center =
            Vector3::new(self.rotation_center.0, self.rotation_center.1, 0.0);

        Matrix4::new_nonuniform_scaling(&Vector3::new(self.scale.0, self.scale.1, 1.0))
            * Matrix4::new_translation(&Vector3::new(self.translation.0, self.translation.1, 0.0))
            * Matrix4::new_translation(&translate_to_rotation_center.clone())
            * Matrix4::new_rotation(Vector3::new(0.0, 0.0, self.angle.to_radians()))
            * Matrix4::new_translation(&-&translate_to_rotation_center)
    }
}
