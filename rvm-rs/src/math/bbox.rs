use glam::Vec3;
use super::Mat3x4f;

#[derive(Debug, Clone, Copy)]
pub struct BBox3f {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for BBox3f {
    fn default() -> Self {
        Self::empty()
    }
}

impl BBox3f {
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.max.x < self.min.x
    }

    pub fn is_not_empty(&self) -> bool {
        self.min.x <= self.max.x
    }

    pub fn engulf_point(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn engulf(&mut self, other: &BBox3f) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn diagonal(&self) -> f32 {
        self.min.distance(self.max)
    }

    pub fn max_side_length(&self) -> f32 {
        let l = self.max - self.min;
        l.x.max(l.y).max(l.z)
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn from_data(data: &[f32; 6]) -> Self {
        Self {
            min: Vec3::new(data[0], data[1], data[2]),
            max: Vec3::new(data[3], data[4], data[5]),
        }
    }

    pub fn transform(m: &Mat3x4f, bbox: &BBox3f) -> BBox3f {
        let mut result = BBox3f::empty();
        let corners = [
            Vec3::new(bbox.min.x, bbox.min.y, bbox.min.z),
            Vec3::new(bbox.max.x, bbox.min.y, bbox.min.z),
            Vec3::new(bbox.min.x, bbox.max.y, bbox.min.z),
            Vec3::new(bbox.max.x, bbox.max.y, bbox.min.z),
            Vec3::new(bbox.min.x, bbox.min.y, bbox.max.z),
            Vec3::new(bbox.max.x, bbox.min.y, bbox.max.z),
            Vec3::new(bbox.min.x, bbox.max.y, bbox.max.z),
            Vec3::new(bbox.max.x, bbox.max.y, bbox.max.z),
        ];
        for c in &corners {
            result.engulf_point(m.transform_point(*c));
        }
        result
    }
}
