use glam::Vec3;
use super::geometry::GeometryId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFlags {
    None = 0,
    HasCircularSide = 1,
    HasRectangularSide = 2,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub geo: [Option<GeometryId>; 2],
    pub offset: [usize; 2],
    pub p: Vec3,
    pub d: Vec3,
    pub temp: u32,
    pub flags: u8,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            geo: [None; 2],
            offset: [0; 2],
            p: Vec3::ZERO,
            d: Vec3::ZERO,
            temp: 0,
            flags: 0,
        }
    }
}

impl Connection {
    pub fn set_flag(&mut self, flag: ConnectionFlags) {
        self.flags |= flag as u8;
    }

    pub fn has_flag(&self, flag: ConnectionFlags) -> bool {
        self.flags & (flag as u8) != 0
    }
}
