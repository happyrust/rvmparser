pub mod node;
pub mod geometry;
pub mod connection;

use std::collections::HashMap;
use node::{Node, NodeKind, NodeId, Attribute};
use geometry::{Geometry, GeometryId};
use connection::{Connection, ConnectionId};

#[derive(Debug, Clone)]
pub struct Color {
    pub color_kind: u32,
    pub color_index: u32,
    pub rgb: [u8; 3],
}

#[derive(Debug)]
pub struct Stats {
    pub group_n: u32,
    pub geometry_n: u32,
    pub pyramid_n: u32,
    pub box_n: u32,
    pub rectangular_torus_n: u32,
    pub circular_torus_n: u32,
    pub elliptical_dish_n: u32,
    pub spherical_dish_n: u32,
    pub snout_n: u32,
    pub cylinder_n: u32,
    pub sphere_n: u32,
    pub line_n: u32,
    pub facetgroup_n: u32,
    pub facetgroup_triangles_n: u32,
    pub facetgroup_quads_n: u32,
    pub facetgroup_polygon_n: u32,
    pub facetgroup_polygon_n_contours_n: u32,
    pub facetgroup_polygon_n_vertices_n: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            group_n: 0, geometry_n: 0,
            pyramid_n: 0, box_n: 0, rectangular_torus_n: 0, circular_torus_n: 0,
            elliptical_dish_n: 0, spherical_dish_n: 0, snout_n: 0, cylinder_n: 0,
            sphere_n: 0, line_n: 0, facetgroup_n: 0,
            facetgroup_triangles_n: 0, facetgroup_quads_n: 0, facetgroup_polygon_n: 0,
            facetgroup_polygon_n_contours_n: 0, facetgroup_polygon_n_vertices_n: 0,
        }
    }
}

pub struct Store {
    pub nodes: Vec<Node>,
    pub geometries: Vec<Geometry>,
    pub connections: Vec<Connection>,
    pub strings: StringInterner,
    pub root_ids: Vec<NodeId>,
    pub stats: Option<Stats>,
    pub colors: Vec<Color>,
    error_string: Option<String>,

    num_groups: u32,
    num_geometries: u32,
}

impl Store {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            geometries: Vec::new(),
            connections: Vec::new(),
            strings: StringInterner::new(),
            root_ids: Vec::new(),
            stats: None,
            colors: Vec::new(),
            error_string: None,
            num_groups: 0,
            num_geometries: 0,
        }
    }

    pub fn new_node(&mut self, parent: Option<NodeId>, kind: NodeKind) -> NodeId {
        let id = NodeId(self.nodes.len());
        let node = Node {
            kind,
            parent,
            children: Vec::new(),
            attributes: Vec::new(),
            ..Default::default()
        };
        self.nodes.push(node);
        if let Some(pid) = parent {
            self.nodes[pid.0].children.push(id);
        } else {
            self.root_ids.push(id);
        }
        id
    }

    pub fn new_geometry(&mut self, parent: NodeId) -> GeometryId {
        let id = GeometryId(self.geometries.len());
        let geo = Geometry {
            parent,
            ..Default::default()
        };
        self.geometries.push(geo);
        self.nodes[parent.0].geometry_ids.push(id);
        id
    }

    pub fn new_connection(&mut self) -> ConnectionId {
        let id = ConnectionId(self.connections.len());
        self.connections.push(Connection::default());
        id
    }

    pub fn new_color(&mut self, color: Color) {
        self.colors.push(color);
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    pub fn geometry(&self, id: GeometryId) -> &Geometry {
        &self.geometries[id.0]
    }

    pub fn geometry_mut(&mut self, id: GeometryId) -> &mut Geometry {
        &mut self.geometries[id.0]
    }

    pub fn connection(&self, id: ConnectionId) -> &Connection {
        &self.connections[id.0]
    }

    pub fn connection_mut(&mut self, id: ConnectionId) -> &mut Connection {
        &mut self.connections[id.0]
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_string = Some(msg);
    }

    pub fn error_string(&self) -> Option<&str> {
        self.error_string.as_deref()
    }

    pub fn group_count(&self) -> u32 {
        self.num_groups
    }

    pub fn geometry_count(&self) -> u32 {
        self.num_geometries
    }

    pub fn update_counts(&mut self) {
        self.num_groups = 0;
        self.num_geometries = 0;
        for root_id in self.root_ids.clone() {
            self.update_counts_recurse(root_id);
        }
    }

    fn update_counts_recurse(&mut self, node_id: NodeId) {
        let node = &self.nodes[node_id.0];
        if matches!(node.kind, NodeKind::Group) {
            self.num_groups += 1;
            self.num_geometries += node.geometry_ids.len() as u32;
        }
        let children: Vec<NodeId> = node.children.clone();
        for child_id in children {
            self.update_counts_recurse(child_id);
        }
    }

    pub fn clone_node(&mut self, parent: Option<NodeId>, src_id: NodeId) -> NodeId {
        let src = &self.nodes[src_id.0];
        let kind = src.kind.clone();
        let file_info = src.file_info.clone();
        let model_info = src.model_info.clone();
        let group_info = src.group_info.clone();
        let attributes: Vec<Attribute> = src.attributes.clone();

        let new_id = self.new_node(parent, kind);
        let node = &mut self.nodes[new_id.0];
        node.file_info = file_info;
        node.model_info = model_info;
        node.group_info = group_info;
        node.attributes = attributes;
        new_id
    }

    pub fn clone_geometry(&mut self, parent: NodeId, src_id: GeometryId) -> GeometryId {
        let src = &self.geometries[src_id.0].clone();
        let new_id = self.new_geometry(parent);
        let geo = &mut self.geometries[new_id.0];
        geo.kind = src.kind.clone();
        geo.geo_type = src.geo_type;
        geo.m_3x4 = src.m_3x4;
        geo.bbox_local = src.bbox_local;
        geo.bbox_world = src.bbox_world;
        geo.transparency = src.transparency;
        geo.color = src.color;
        geo.color_name = src.color_name.clone();
        geo.sample_start_angle = src.sample_start_angle;
        new_id
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StringInterner {
    pub map: HashMap<String, usize>,
    pub strings: Vec<String>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            strings: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.map.get(s) {
            id
        } else {
            let id = self.strings.len();
            self.strings.push(s.to_string());
            self.map.insert(s.to_string(), id);
            id
        }
    }

    pub fn get(&self, id: usize) -> &str {
        &self.strings[id]
    }

    pub fn get_id(&self, s: &str) -> Option<usize> {
        self.map.get(s).copied()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
