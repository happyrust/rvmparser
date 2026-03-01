use std::collections::HashSet;
use crate::store::Store;
use crate::store::node::NodeId;

pub fn discard_groups(store: &mut Store, tag_text: &str) -> u32 {
    let mut discard_tags = HashSet::new();

    for line in tag_text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let name = if let Some(pos) = line.rfind('\t') {
            &line[pos+1..]
        } else {
            line
        };
        let name_id = store.strings.intern(name);
        discard_tags.insert(name_id);
    }

    let mut discarded = 0u32;
    let root_ids: Vec<NodeId> = store.root_ids.clone();
    for root_id in root_ids {
        let model_ids: Vec<NodeId> = store.node(root_id).children.clone();
        for model_id in model_ids {
            prune_children(store, model_id, &discard_tags, &mut discarded);
        }
    }

    discarded
}

fn prune_children(store: &mut Store, parent_id: NodeId, discard_tags: &HashSet<usize>, discarded: &mut u32) {
    let children: Vec<NodeId> = store.node(parent_id).children.clone();
    let mut kept = Vec::new();

    for child_id in children {
        let should_discard = store.node(child_id).group_info.as_ref()
            .map(|gi| discard_tags.contains(&gi.name))
            .unwrap_or(false);

        if should_discard {
            *discarded += 1;
        } else {
            prune_children(store, child_id, discard_tags, discarded);
            kept.push(child_id);
        }
    }

    store.node_mut(parent_id).children = kept;
}
