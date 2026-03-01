pub mod bbox;

use glam::{Vec3, Mat3, Mat4};

#[derive(Debug, Clone, Copy)]
pub struct Mat3x4f {
    pub data: [f32; 12],
}

impl Default for Mat3x4f {
    fn default() -> Self {
        Self { data: [0.0; 12] }
    }
}

impl Mat3x4f {
    pub fn mat3(&self) -> Mat3 {
        Mat3::from_cols(
            Vec3::new(self.data[0], self.data[1], self.data[2]),
            Vec3::new(self.data[3], self.data[4], self.data[5]),
            Vec3::new(self.data[6], self.data[7], self.data[8]),
        )
    }

    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.data[9], self.data[10], self.data[11])
    }

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let m = self.mat3();
        m * p + self.translation()
    }

    pub fn transform_dir(&self, d: Vec3) -> Vec3 {
        self.mat3() * d
    }

    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_cols(
            self.mat3().x_axis.extend(0.0),
            self.mat3().y_axis.extend(0.0),
            self.mat3().z_axis.extend(0.0),
            self.translation().extend(1.0),
        )
    }
}

pub fn get_scale(m: &Mat3x4f) -> f32 {
    let n = m.mat3();
    let c0 = n.x_axis.length();
    let c1 = n.y_axis.length();
    let c2 = n.z_axis.length();
    let mut s = c0;
    if c1 > s { s = c1; }
    if c2 > s { s = c2; }
    s
}
