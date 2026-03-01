use std::io::Write;
use serde_json::{json, Value};
use crate::store::Store;
use crate::store::node::{NodeId, NodeKind};
use crate::store::geometry::GeometryId;
use crate::export::tessellator;
use super::ExportError;

pub struct GltfExportOptions {
    pub tolerance: f32,
    pub rotate_z_to_y: bool,
    pub center: bool,
    pub include_attributes: bool,
    pub merge_geos: bool,
    pub split_level: usize,
}

impl Default for GltfExportOptions {
    fn default() -> Self {
        Self {
            tolerance: 0.1,
            rotate_z_to_y: true,
            center: false,
            include_attributes: true,
            merge_geos: true,
            split_level: 0,
        }
    }
}

struct GltfBuilder {
    nodes: Vec<Value>,
    meshes: Vec<Value>,
    accessors: Vec<Value>,
    buffer_views: Vec<Value>,
    materials: Vec<Value>,
    buffer_data: Vec<u8>,
    material_map: std::collections::HashMap<u32, usize>,
}

impl GltfBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            meshes: Vec::new(),
            accessors: Vec::new(),
            buffer_views: Vec::new(),
            materials: Vec::new(),
            buffer_data: Vec::new(),
            material_map: std::collections::HashMap::new(),
        }
    }

    fn get_or_create_material(&mut self, color: u32) -> usize {
        if let Some(&idx) = self.material_map.get(&color) {
            return idx;
        }
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;
        let idx = self.materials.len();
        self.materials.push(json!({
            "pbrMetallicRoughness": {
                "baseColorFactor": [r, g, b, 1.0],
                "metallicFactor": 0.0,
                "roughnessFactor": 0.8
            }
        }));
        self.material_map.insert(color, idx);
        idx
    }

    fn add_buffer_view(&mut self, data: &[u8], target: u32) -> usize {
        let offset = self.buffer_data.len();
        self.buffer_data.extend_from_slice(data);
        // Pad to 4-byte alignment
        while self.buffer_data.len() % 4 != 0 {
            self.buffer_data.push(0);
        }
        let idx = self.buffer_views.len();
        self.buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": data.len(),
            "target": target,
        }));
        idx
    }

    fn add_accessor(&mut self, buffer_view: usize, component_type: u32, count: u32, type_str: &str, min: Option<Vec<f32>>, max: Option<Vec<f32>>) -> usize {
        let idx = self.accessors.len();
        let mut acc = json!({
            "bufferView": buffer_view,
            "componentType": component_type,
            "count": count,
            "type": type_str,
        });
        if let Some(min_val) = min {
            acc["min"] = json!(min_val);
        }
        if let Some(max_val) = max {
            acc["max"] = json!(max_val);
        }
        self.accessors.push(acc);
        idx
    }
}

pub fn export_gltf(
    store: &Store,
    path: &str,
    options: &GltfExportOptions,
) -> Result<(), ExportError> {
    let is_glb = path.ends_with(".glb");
    let mut builder = GltfBuilder::new();

    let scene_children = build_gltf_nodes(store, &mut builder, options);

    let root_children = if options.rotate_z_to_y {
        let rot_node = json!({
            "name": "Z-to-Y rotation",
            "rotation": [-0.7071067811865476, 0.0, 0.0, 0.7071067811865476],
            "children": scene_children,
        });
        let rot_idx = builder.nodes.len();
        builder.nodes.push(rot_node);
        vec![rot_idx]
    } else {
        scene_children
    };

    let buffer_len = builder.buffer_data.len();

    let mut gltf = json!({
        "asset": { "version": "2.0", "generator": "rvmparser-rs" },
        "scene": 0,
        "scenes": [{ "nodes": root_children }],
        "nodes": builder.nodes,
    });

    if !builder.meshes.is_empty() {
        gltf["meshes"] = json!(builder.meshes);
    }
    if !builder.accessors.is_empty() {
        gltf["accessors"] = json!(builder.accessors);
    }
    if !builder.buffer_views.is_empty() {
        gltf["bufferViews"] = json!(builder.buffer_views);
    }
    if !builder.materials.is_empty() {
        gltf["materials"] = json!(builder.materials);
    }

    if is_glb {
        let json_str = serde_json::to_string(&gltf)
            .map_err(|e| ExportError::Other(e.to_string()))?;
        let mut json_bytes = json_str.into_bytes();
        while json_bytes.len() % 4 != 0 { json_bytes.push(0x20); }

        let mut bin_data = builder.buffer_data;
        while bin_data.len() % 4 != 0 { bin_data.push(0); }

        let total_len = 12 + 8 + json_bytes.len() + 8 + bin_data.len();

        let mut file = std::fs::File::create(path)?;
        file.write_all(b"glTF")?;
        file.write_all(&2u32.to_le_bytes())?;
        file.write_all(&(total_len as u32).to_le_bytes())?;

        file.write_all(&(json_bytes.len() as u32).to_le_bytes())?;
        file.write_all(&0x4E4F534Au32.to_le_bytes())?; // JSON
        file.write_all(&json_bytes)?;

        file.write_all(&(bin_data.len() as u32).to_le_bytes())?;
        file.write_all(&0x004E4942u32.to_le_bytes())?; // BIN
        file.write_all(&bin_data)?;
    } else {
        gltf["buffers"] = json!([{
            "byteLength": buffer_len,
            "uri": format!("data:application/octet-stream;base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &builder.buffer_data)),
        }]);

        let json_str = serde_json::to_string_pretty(&gltf)
            .map_err(|e| ExportError::Other(e.to_string()))?;
        std::fs::write(path, json_str)?;
    }

    Ok(())
}

fn build_gltf_nodes(
    store: &Store,
    builder: &mut GltfBuilder,
    options: &GltfExportOptions,
) -> Vec<usize> {
    let mut scene_children = Vec::new();
    for &root_id in &store.root_ids {
        if let Some(idx) = build_gltf_node(store, builder, root_id, options) {
            scene_children.push(idx);
        }
    }
    scene_children
}

fn build_gltf_node(
    store: &Store,
    builder: &mut GltfBuilder,
    node_id: NodeId,
    options: &GltfExportOptions,
) -> Option<usize> {
    let node = store.node(node_id);
    let mut gltf_node = serde_json::Map::new();

    match node.kind {
        NodeKind::File => {
            if let Some(ref fi) = node.file_info {
                gltf_node.insert("name".into(), json!(store.strings.get(fi.info)));
            }
        }
        NodeKind::Model => {
            if let Some(ref mi) = node.model_info {
                gltf_node.insert("name".into(), json!(store.strings.get(mi.name)));
            }
        }
        NodeKind::Group => {
            if let Some(ref gi) = node.group_info {
                gltf_node.insert("name".into(), json!(store.strings.get(gi.name)));
            }

            if options.include_attributes && !node.attributes.is_empty() {
                let mut extras = serde_json::Map::new();
                for attr in &node.attributes {
                    extras.insert(
                        store.strings.get(attr.key).to_string(),
                        json!(store.strings.get(attr.val)),
                    );
                }
                gltf_node.insert("extras".into(), Value::Object(extras));
            }

            let mut primitives = Vec::new();
            for &geo_id in &node.geometry_ids {
                if let Some(prim) = build_gltf_primitive(store, builder, geo_id, options) {
                    primitives.push(prim);
                }
            }

            if !primitives.is_empty() {
                let mesh_idx = builder.meshes.len();
                builder.meshes.push(json!({ "primitives": primitives }));
                gltf_node.insert("mesh".into(), json!(mesh_idx));
            }
        }
    }

    let mut child_indices = Vec::new();
    for &child_id in &node.children {
        if let Some(idx) = build_gltf_node(store, builder, child_id, options) {
            child_indices.push(idx);
        }
    }

    if !child_indices.is_empty() {
        gltf_node.insert("children".into(), json!(child_indices));
    }

    if gltf_node.is_empty() && child_indices.is_empty() {
        return None;
    }

    let idx = builder.nodes.len();
    builder.nodes.push(Value::Object(gltf_node));
    Some(idx)
}

fn build_gltf_primitive(
    store: &Store,
    builder: &mut GltfBuilder,
    geo_id: GeometryId,
    options: &GltfExportOptions,
) -> Option<Value> {
    let geo = store.geometry(geo_id);
    let tri = geo.triangulation.clone()
        .or_else(|| tessellator::tessellate(store, geo_id, options.tolerance))?;

    if tri.triangles_n == 0 { return None; }

    let m = &geo.m_3x4;
    let mut positions: Vec<f32> = Vec::with_capacity(tri.vertices_n as usize * 3);
    let mut normals: Vec<f32> = Vec::with_capacity(tri.vertices_n as usize * 3);

    let mut p_min = [f32::MAX; 3];
    let mut p_max = [f32::MIN; 3];

    for i in 0..tri.vertices_n as usize {
        let lp = glam::Vec3::new(tri.vertices[3*i], tri.vertices[3*i+1], tri.vertices[3*i+2]);
        let ln = glam::Vec3::new(tri.normals[3*i], tri.normals[3*i+1], tri.normals[3*i+2]);
        let wp = m.transform_point(lp);
        let wn = m.transform_dir(ln).normalize_or_zero();
        positions.extend_from_slice(&[wp.x, wp.y, wp.z]);
        normals.extend_from_slice(&[wn.x, wn.y, wn.z]);
        for k in 0..3 {
            p_min[k] = p_min[k].min(positions[3*i + k]);
            p_max[k] = p_max[k].max(positions[3*i + k]);
        }
    }

    let pos_bytes: Vec<u8> = positions.iter().flat_map(|f| f.to_le_bytes()).collect();
    let norm_bytes: Vec<u8> = normals.iter().flat_map(|f| f.to_le_bytes()).collect();
    let idx_bytes: Vec<u8> = tri.indices.iter().flat_map(|i| i.to_le_bytes()).collect();

    let pos_bv = builder.add_buffer_view(&pos_bytes, 34962);
    let norm_bv = builder.add_buffer_view(&norm_bytes, 34962);
    let idx_bv = builder.add_buffer_view(&idx_bytes, 34963);

    let pos_acc = builder.add_accessor(pos_bv, 5126, tri.vertices_n, "VEC3",
        Some(p_min.to_vec()), Some(p_max.to_vec()));
    let norm_acc = builder.add_accessor(norm_bv, 5126, tri.vertices_n, "VEC3", None, None);
    let idx_acc = builder.add_accessor(idx_bv, 5125, tri.triangles_n * 3, "SCALAR", None, None);

    let mat_idx = builder.get_or_create_material(geo.color);

    Some(json!({
        "attributes": {
            "POSITION": pos_acc,
            "NORMAL": norm_acc,
        },
        "indices": idx_acc,
        "material": mat_idx,
    }))
}
