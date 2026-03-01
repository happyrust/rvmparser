use std::io::Write;
use crate::store::Store;
use crate::store::node::{NodeId, NodeKind};
use crate::store::geometry::GeometryId;
use crate::export::tessellator;
use super::ExportError;

pub struct ObjExportOptions {
    pub tolerance: f32,
    pub group_bounding_boxes: bool,
}

impl Default for ObjExportOptions {
    fn default() -> Self {
        Self { tolerance: 0.1, group_bounding_boxes: false }
    }
}

pub fn export_obj(
    store: &Store,
    obj_path: &str,
    mtl_path: &str,
    options: &ObjExportOptions,
) -> Result<(), ExportError> {
    let mut obj_file = std::fs::File::create(obj_path)?;
    let mut mtl_file = std::fs::File::create(mtl_path)?;

    let mtl_name = std::path::Path::new(mtl_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("material.mtl");

    writeln!(obj_file, "# RVM Parser Rust - OBJ Export")?;
    writeln!(obj_file, "mtllib {}", mtl_name)?;

    writeln!(mtl_file, "# RVM Parser Rust - MTL Export")?;

    let mut vertex_offset = 1u32;
    let mut color_set = std::collections::HashSet::new();

    for &root_id in &store.root_ids {
        export_node_obj(store, root_id, &mut obj_file, &mut mtl_file, &mut vertex_offset, &mut color_set, options)?;
    }

    Ok(())
}

fn export_node_obj(
    store: &Store,
    node_id: NodeId,
    obj_file: &mut std::fs::File,
    mtl_file: &mut std::fs::File,
    vertex_offset: &mut u32,
    color_set: &mut std::collections::HashSet<u32>,
    options: &ObjExportOptions,
) -> Result<(), ExportError> {
    let node = store.node(node_id);

    if matches!(node.kind, NodeKind::Group) {
        let name = node.group_info.as_ref()
            .map(|g| store.strings.get(g.name).to_string())
            .unwrap_or_default();
        writeln!(obj_file, "g {}", name)?;

        for &geo_id in &node.geometry_ids {
            export_geometry_obj(store, geo_id, obj_file, mtl_file, vertex_offset, color_set, options)?;
        }
    }

    let children: Vec<NodeId> = node.children.clone();
    for child_id in children {
        export_node_obj(store, child_id, obj_file, mtl_file, vertex_offset, color_set, options)?;
    }

    Ok(())
}

fn export_geometry_obj(
    store: &Store,
    geo_id: GeometryId,
    obj_file: &mut std::fs::File,
    mtl_file: &mut std::fs::File,
    vertex_offset: &mut u32,
    color_set: &mut std::collections::HashSet<u32>,
    options: &ObjExportOptions,
) -> Result<(), ExportError> {
    let geo = store.geometry(geo_id);

    let tri = match &geo.triangulation {
        Some(t) => t.clone(),
        None => match tessellator::tessellate(store, geo_id, options.tolerance) {
            Some(t) => t,
            None => return Ok(()),
        }
    };

    if tri.triangles_n == 0 { return Ok(()); }

    let color = geo.color;
    let mat_name = format!("mat_{:06x}", color);

    if color_set.insert(color) {
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;
        writeln!(mtl_file, "newmtl {}", mat_name)?;
        writeln!(mtl_file, "Kd {} {} {}", r, g, b)?;
        writeln!(mtl_file)?;
    }

    writeln!(obj_file, "usemtl {}", mat_name)?;

    let m = &geo.m_3x4;
    for i in 0..tri.vertices_n as usize {
        let lx = tri.vertices[3 * i];
        let ly = tri.vertices[3 * i + 1];
        let lz = tri.vertices[3 * i + 2];
        let p = m.transform_point(glam::Vec3::new(lx, ly, lz));
        writeln!(obj_file, "v {} {} {}", p.x, p.y, p.z)?;
    }

    for i in 0..tri.vertices_n as usize {
        let nx = tri.normals[3 * i];
        let ny = tri.normals[3 * i + 1];
        let nz = tri.normals[3 * i + 2];
        let n = m.transform_dir(glam::Vec3::new(nx, ny, nz)).normalize_or_zero();
        writeln!(obj_file, "vn {} {} {}", n.x, n.y, n.z)?;
    }

    for i in 0..tri.triangles_n as usize {
        let a = tri.indices[3 * i] + *vertex_offset;
        let b = tri.indices[3 * i + 1] + *vertex_offset;
        let c = tri.indices[3 * i + 2] + *vertex_offset;
        writeln!(obj_file, "f {}//{} {}//{} {}//{}", a, a, b, b, c, c)?;
    }

    *vertex_offset += tri.vertices_n;
    Ok(())
}
