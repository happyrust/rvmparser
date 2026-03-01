use crate::store::Store;
use crate::store::node::NodeId;
use crate::store::geometry::GeometryKind;
use crate::hierarchy::flatten::Flatten;

struct StackItem {
    _node_id: NodeId,
    _name_id: usize,
    vertices: u32,
    keep: bool,
}

pub fn chunk_tiny(store: &Store, flatten: &mut Flatten, vertex_threshold: u32) {
    let root_ids: Vec<NodeId> = store.root_ids.clone();
    for root_id in &root_ids {
        let model_ids: Vec<NodeId> = store.node(*root_id).children.clone();
        for model_id in &model_ids {
            let group_ids: Vec<NodeId> = store.node(*model_id).children.clone();
            for group_id in &group_ids {
                chunk_tiny_recurse(store, flatten, *group_id, vertex_threshold, &mut Vec::new());
            }
        }
    }
}

fn chunk_tiny_recurse(
    store: &Store,
    flatten: &mut Flatten,
    node_id: NodeId,
    vertex_threshold: u32,
    stack: &mut Vec<StackItem>,
) -> u32 {
    let name_id = store.node(node_id).group_info.as_ref()
        .map(|gi| gi.name)
        .unwrap_or(0);

    let mut vertices = 0u32;

    let geo_ids = store.node(node_id).geometry_ids.clone();
    for geo_id in &geo_ids {
        let geo = store.geometry(*geo_id);
        if matches!(geo.kind, GeometryKind::Line { .. }) {
            vertices += 2;
        } else if let Some(ref tri) = geo.triangulation {
            vertices += tri.vertices_n;
        }
    }

    stack.push(StackItem { _node_id: node_id, _name_id: name_id, vertices: 0, keep: false });
    let stack_idx = stack.len() - 1;
    stack[stack_idx].vertices = vertices;

    let children: Vec<NodeId> = store.node(node_id).children.clone();
    for child_id in children {
        let child_verts = chunk_tiny_recurse(store, flatten, child_id, vertex_threshold, stack);
        vertices += child_verts;
    }

    let should_keep = stack.len() == 1
        || (stack.len() >= 2 && stack[stack.len() - 2].keep)
        || vertex_threshold < vertices;

    if should_keep {
        flatten.keep_tag(name_id);
        for item in stack.iter_mut() {
            item.keep = true;
        }
    } else if stack.len() >= 2 {
        let idx = stack.len() - 2;
        stack[idx].vertices += vertices;
    }

    stack.pop();
    vertices
}
