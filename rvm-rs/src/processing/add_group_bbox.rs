use crate::store::Store;
use crate::store::node::NodeId;
use crate::math::bbox::BBox3f;

pub fn add_group_bboxes(store: &mut Store) {
    let root_ids: Vec<NodeId> = store.root_ids.clone();
    for root_id in root_ids {
        let model_ids: Vec<NodeId> = store.node(root_id).children.clone();
        for model_id in model_ids {
            let group_ids: Vec<NodeId> = store.node(model_id).children.clone();
            for group_id in group_ids {
                compute_bbox_recurse(store, group_id);
            }
        }
    }
}

fn compute_bbox_recurse(store: &mut Store, node_id: NodeId) -> BBox3f {
    let mut bbox = BBox3f::empty();

    let geo_ids = store.node(node_id).geometry_ids.clone();
    for geo_id in geo_ids {
        let geo = store.geometry(geo_id);
        bbox.engulf(&geo.bbox_world);
    }

    let children: Vec<NodeId> = store.node(node_id).children.clone();
    for child_id in children {
        let child_bbox = compute_bbox_recurse(store, child_id);
        if child_bbox.is_not_empty() {
            bbox.engulf(&child_bbox);
        }
    }

    if let Some(ref mut gi) = store.node_mut(node_id).group_info {
        gi.bbox_world = bbox;
    }

    bbox
}
