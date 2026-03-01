use std::io::Write;
use crate::store::Store;
use crate::store::node::{NodeId, NodeKind};
use crate::store::geometry::GeometryKind;

pub fn dump_names(store: &Store, path: &str) -> Result<(), std::io::Error> {
    let mut out = std::fs::File::create(path)?;

    for &root_id in &store.root_ids {
        dump_node(store, &mut out, root_id, 0)?;
    }

    Ok(())
}

fn dump_node(store: &Store, out: &mut impl Write, node_id: NodeId, depth: usize) -> Result<(), std::io::Error> {
    let node = store.node(node_id);

    match node.kind {
        NodeKind::File => {
            writeln!(out, "File:")?;
            if let Some(ref fi) = node.file_info {
                writeln!(out, "    info:     \"{}\"", store.strings.get(fi.info))?;
                writeln!(out, "    note:     \"{}\"", store.strings.get(fi.note))?;
                writeln!(out, "    date:     \"{}\"", store.strings.get(fi.date))?;
                writeln!(out, "    user:     \"{}\"", store.strings.get(fi.user))?;
                writeln!(out, "    encoding: \"{}\"", store.strings.get(fi.encoding))?;
            }
        }
        NodeKind::Model => {
            writeln!(out, "Model:")?;
            if let Some(ref mi) = node.model_info {
                writeln!(out, "    project:  \"{}\"", store.strings.get(mi.project))?;
                writeln!(out, "    name:     \"{}\"", store.strings.get(mi.name))?;
            }
        }
        NodeKind::Group => {
            let name = node.group_info.as_ref()
                .map(|gi| store.strings.get(gi.name).to_string())
                .unwrap_or_default();

            let indent: String = "    ".repeat(depth);
            write!(out, "{}{}", indent, name)?;

            let mut pgeos = 0u32;
            let mut fgrps = 0u32;
            for &geo_id in &node.geometry_ids {
                let geo = store.geometry(geo_id);
                if matches!(geo.kind, GeometryKind::FacetGroup { .. }) {
                    fgrps += 1;
                } else {
                    pgeos += 1;
                }
            }

            writeln!(out)?;
            if pgeos > 0 {
                writeln!(out, "{} pgeos={}", indent, pgeos)?;
            }
            if fgrps > 0 {
                writeln!(out, "{} fgrps={}", indent, fgrps)?;
            }
        }
    }

    let children: Vec<NodeId> = node.children.clone();
    for child_id in children {
        dump_node(store, out, child_id, depth + 1)?;
    }

    Ok(())
}
