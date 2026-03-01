use crate::math::Mat3x4f;
use crate::math::bbox::BBox3f;
use super::node::NodeId;
use super::connection::ConnectionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometryId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    Primitive,
    Obstruction,
    Insulation,
}

impl Default for GeometryType {
    fn default() -> Self { Self::Primitive }
}

#[derive(Debug, Clone)]
pub struct Contour {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub vertices_n: u32,
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub contours: Vec<Contour>,
}

#[derive(Debug, Clone)]
pub struct Triangulation {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertices_n: u32,
    pub triangles_n: u32,
    pub error: f32,
}

#[derive(Debug, Clone)]
pub enum GeometryKind {
    Pyramid {
        bottom: [f32; 2],
        top: [f32; 2],
        offset: [f32; 2],
        height: f32,
    },
    Box {
        lengths: [f32; 3],
    },
    RectangularTorus {
        inner_radius: f32,
        outer_radius: f32,
        height: f32,
        angle: f32,
    },
    CircularTorus {
        offset: f32,
        radius: f32,
        angle: f32,
    },
    EllipticalDish {
        base_radius: f32,
        height: f32,
    },
    SphericalDish {
        base_radius: f32,
        height: f32,
    },
    Snout {
        offset: [f32; 2],
        bshear: [f32; 2],
        tshear: [f32; 2],
        radius_b: f32,
        radius_t: f32,
        height: f32,
    },
    Cylinder {
        radius: f32,
        height: f32,
    },
    Sphere {
        diameter: f32,
    },
    Line {
        a: f32,
        b: f32,
    },
    FacetGroup {
        polygons: Vec<Polygon>,
    },
}

impl Default for GeometryKind {
    fn default() -> Self {
        GeometryKind::Box { lengths: [0.0; 3] }
    }
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub parent: NodeId,
    pub kind: GeometryKind,
    pub geo_type: GeometryType,
    pub m_3x4: Mat3x4f,
    pub bbox_local: BBox3f,
    pub bbox_world: BBox3f,
    pub transparency: u32,
    pub color: u32,
    pub color_name: Option<String>,
    pub sample_start_angle: f32,
    pub triangulation: Option<Triangulation>,
    pub connections: [Option<ConnectionId>; 6],
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            parent: NodeId(0),
            kind: GeometryKind::default(),
            geo_type: GeometryType::default(),
            m_3x4: Mat3x4f::default(),
            bbox_local: BBox3f::empty(),
            bbox_world: BBox3f::empty(),
            transparency: 0,
            color: 0x202020,
            color_name: None,
            sample_start_angle: 0.0,
            triangulation: None,
            connections: [None; 6],
        }
    }
}
