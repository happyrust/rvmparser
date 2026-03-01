use regex::Regex;
use crate::store::Store;
use crate::store::node::{NodeId, Attribute};
use crate::store::geometry::GeometryId;

pub fn flatten_regex(store: &mut Store, pattern: &str) -> Result<(), String> {
    let re = Regex::new(pattern).map_err(|e| format!("Failed to compile regex '{}': {}", pattern, e))?;

    let root_ids: Vec<NodeId> = store.root_ids.clone();
    for root_id in root_ids {
        let model_ids: Vec<NodeId> = store.node(root_id).children.clone();
        for model_id in model_ids {
            let group_ids: Vec<NodeId> = store.node(model_id).children.clone();
            for group_id in group_ids {
                handle_children(store, &re, group_id, group_id);
            }
        }
    }

    Ok(())
}

fn handle_children(store: &mut Store, re: &Regex, nearest_kept: NodeId, parent: NodeId) {
    let children: Vec<NodeId> = store.node(parent).children.clone();
    store.node_mut(parent).children.clear();

    for child_id in children {
        let name = store.node(child_id).group_info.as_ref()
            .map(|gi| store.strings.get(gi.name).to_string())
            .unwrap_or_default();

        if re.is_match(&name) {
            store.node_mut(nearest_kept).children.push(child_id);
            handle_children(store, re, child_id, child_id);
        } else {
            // Move attributes from discarded node to nearest kept ancestor
            let attrs: Vec<Attribute> = store.node(child_id).attributes.clone();
            store.node_mut(nearest_kept).attributes.extend(attrs);

            // Move geometries from discarded node to nearest kept ancestor
            let geo_ids: Vec<GeometryId> = store.node(child_id).geometry_ids.clone();
            store.node_mut(nearest_kept).geometry_ids.extend(geo_ids);

            handle_children(store, re, nearest_kept, child_id);
        }
    }
}
