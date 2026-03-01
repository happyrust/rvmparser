use std::io::Write;
use crate::store::Store;
use crate::store::node::NodeId;
use crate::store::geometry::{GeometryId, GeometryKind, GeometryType};
use super::ExportError;

pub fn export_rev(store: &Store, path: &str) -> Result<(), ExportError> {
    let mut out = std::fs::File::create(path)?;

    for &root_id in &store.root_ids {
        write_file(store, &mut out, root_id)?;
    }

    Ok(())
}

fn write_chunk_header(out: &mut impl Write, id: &str, u0: u32, u1: u32) -> Result<(), ExportError> {
    writeln!(out, "{}", id)?;
    writeln!(out, "{:6}{:6}", u0, u1)?;
    Ok(())
}

fn write_file(store: &Store, out: &mut impl Write, node_id: NodeId) -> Result<(), ExportError> {
    let node = store.node(node_id);
    let fi = node.file_info.as_ref().ok_or_else(|| ExportError::Other("Not a file node".into()))?;

    write_chunk_header(out, "HEAD", 1, 1)?;
    writeln!(out, "{}", store.strings.get(fi.info))?;
    writeln!(out, "{}", store.strings.get(fi.note))?;
    writeln!(out, "{}", store.strings.get(fi.date))?;
    writeln!(out, "{}", store.strings.get(fi.user))?;

    for &child_id in &node.children {
        write_model(store, out, child_id)?;
    }

    write_chunk_header(out, "END:", 1, 1)?;
    Ok(())
}

fn write_model(store: &Store, out: &mut impl Write, node_id: NodeId) -> Result<(), ExportError> {
    let node = store.node(node_id);
    let mi = node.model_info.as_ref().ok_or_else(|| ExportError::Other("Not a model node".into()))?;

    write_chunk_header(out, "MODL", 1, 1)?;
    writeln!(out, "{}", store.strings.get(mi.project))?;
    writeln!(out, "{}", store.strings.get(mi.name))?;

    for &child_id in &node.children {
        write_group(store, out, child_id)?;
    }

    Ok(())
}

fn write_group(store: &Store, out: &mut impl Write, node_id: NodeId) -> Result<(), ExportError> {
    let node = store.node(node_id);
    let gi = match &node.group_info {
        Some(g) => g,
        None => return Ok(()),
    };

    write_chunk_header(out, "CNTB", 1, 1)?;
    writeln!(out, "{}", store.strings.get(gi.name))?;
    writeln!(out, "{:14.5}{:14.5}{:14.5}",
        1000.0 * gi.translation[0],
        1000.0 * gi.translation[1],
        1000.0 * gi.translation[2])?;
    writeln!(out, "{:6}", gi.material)?;

    for &child_id in &node.children {
        write_group(store, out, child_id)?;
    }

    for &geo_id in &node.geometry_ids {
        write_geometry(store, out, geo_id)?;
    }

    write_chunk_header(out, "CNTE", 1, 1)?;
    Ok(())
}

fn write_geometry(store: &Store, out: &mut impl Write, geo_id: GeometryId) -> Result<(), ExportError> {
    let geo = store.geometry(geo_id);

    if matches!(geo.kind, GeometryKind::Sphere { .. }) {
        eprintln!("[W] SKIPPING unsupported primitive type Sphere in REV export");
        return Ok(());
    }

    let chunk = match geo.geo_type {
        GeometryType::Primitive => "PRIM",
        GeometryType::Obstruction => "OBST",
        GeometryType::Insulation => "INSU",
    };
    write_chunk_header(out, chunk, 1, 1)?;

    let kind_num: u32 = match &geo.kind {
        GeometryKind::Pyramid { .. } => 1,
        GeometryKind::Box { .. } => 2,
        GeometryKind::RectangularTorus { .. } => 3,
        GeometryKind::CircularTorus { .. } => 4,
        GeometryKind::EllipticalDish { .. } => 5,
        GeometryKind::SphericalDish { .. } => 6,
        GeometryKind::Snout { .. } => 7,
        GeometryKind::Cylinder { .. } => 8,
        GeometryKind::Sphere { .. } => 9,
        GeometryKind::Line { .. } => 10,
        GeometryKind::FacetGroup { .. } => 11,
    };
    writeln!(out, "{:6}", kind_num)?;

    // Write 3x4 matrix (column major, as 3 rows of 4 values)
    let d = &geo.m_3x4.data;
    for k in 0..3 {
        writeln!(out, "{:14.5}{:14.5}{:14.5}{:14.5}",
            d[k], d[3 + k], d[6 + k], d[9 + k])?;
    }

    // Write bbox
    let bl = &geo.bbox_local;
    writeln!(out, "{:14.5}{:14.5}{:14.5}", bl.min.x, bl.min.y, bl.min.z)?;
    writeln!(out, "{:14.5}{:14.5}{:14.5}", bl.max.x, bl.max.y, bl.max.z)?;

    match &geo.kind {
        GeometryKind::Pyramid { bottom, top, offset, height } => {
            writeln!(out, "{:14.5}{:14.5}{:14.5}{:14.5}", bottom[0], bottom[1], top[0], top[1])?;
            writeln!(out, "{:14.5}{:14.5}{:14.5}", offset[0], offset[1], height)?;
        }
        GeometryKind::Box { lengths } => {
            writeln!(out, "{:14.5}{:14.5}{:14.5}", lengths[0], lengths[1], lengths[2])?;
        }
        GeometryKind::RectangularTorus { inner_radius, outer_radius, height, angle } => {
            writeln!(out, "{:14.5}{:14.5}{:14.5}{:14.5}", inner_radius, outer_radius, height, angle)?;
        }
        GeometryKind::CircularTorus { offset, radius, angle } => {
            writeln!(out, "{:14.5}{:14.5}{:14.5}", offset, radius, angle)?;
        }
        GeometryKind::EllipticalDish { base_radius, height } => {
            writeln!(out, "{:14.5}{:14.5}", base_radius, height)?;
        }
        GeometryKind::SphericalDish { base_radius, height } => {
            writeln!(out, "{:14.5}{:14.5}", base_radius, height)?;
        }
        GeometryKind::Snout { radius_b, radius_t, height, offset, bshear, tshear } => {
            writeln!(out, "{:14.5}{:14.5}{:14.5}{:14.5}{:14.5}", radius_b, radius_t, height, offset[0], offset[1])?;
            writeln!(out, "{:14.5}{:14.5}{:14.5}{:14.5}", bshear[0], bshear[1], tshear[0], tshear[1])?;
        }
        GeometryKind::Cylinder { radius, height } => {
            writeln!(out, "{:14.5}{:14.5}", radius, height)?;
        }
        GeometryKind::Sphere { .. } => unreachable!(),
        GeometryKind::Line { a, b } => {
            writeln!(out, "{:14.5}{:14.5}", a, b)?;
        }
        GeometryKind::FacetGroup { polygons } => {
            writeln!(out, "{:6}", polygons.len())?;
            for poly in polygons {
                writeln!(out, "{:6}", poly.contours.len())?;
                for cont in &poly.contours {
                    writeln!(out, "{:6}", cont.vertices_n)?;
                    for vi in 0..cont.vertices_n as usize {
                        writeln!(out, "{:14.5}{:14.5}{:14.5}",
                            cont.vertices[3*vi], cont.vertices[3*vi+1], cont.vertices[3*vi+2])?;
                        writeln!(out, "{:14.5}{:14.5}{:14.5}",
                            cont.normals[3*vi], cont.normals[3*vi+1], cont.normals[3*vi+2])?;
                    }
                }
            }
        }
    }

    Ok(())
}
