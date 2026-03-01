use crate::math::bbox::BBox3f;
use super::geometry::GeometryId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    File,
    Model,
    Group,
}

#[derive(Debug, Clone, Default)]
pub struct FileInfo {
    pub info: usize,
    pub note: usize,
    pub date: usize,
    pub user: usize,
    pub encoding: usize,
    pub path: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ModelInfo {
    pub project: usize,
    pub name: usize,
}

#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub name: usize,
    pub bbox_world: BBox3f,
    pub material: u32,
    pub transparency: u32,
    pub id: i32,
    pub translation: [f32; 3],
}

impl Default for GroupInfo {
    fn default() -> Self {
        Self {
            name: 0,
            bbox_world: BBox3f::empty(),
            material: 0,
            transparency: 0,
            id: 0,
            translation: [0.0; 3],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: usize,
    pub val: usize,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub attributes: Vec<Attribute>,
    pub geometry_ids: Vec<GeometryId>,

    pub file_info: Option<FileInfo>,
    pub model_info: Option<ModelInfo>,
    pub group_info: Option<GroupInfo>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            kind: NodeKind::Group,
            parent: None,
            children: Vec::new(),
            attributes: Vec::new(),
            geometry_ids: Vec::new(),
            file_info: None,
            model_info: None,
            group_info: None,
        }
    }
}
