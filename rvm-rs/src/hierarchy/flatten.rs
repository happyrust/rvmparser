use std::collections::HashMap;
use crate::store::Store;
use crate::store::node::NodeId;
use crate::store::geometry::GeometryId;

pub struct Flatten {
    src_tags: HashMap<usize, NodeId>,
    tags: HashMap<usize, i32>,
    current_index: i32,
}

impl Flatten {
    pub fn new(store: &Store) -> Self {
        let mut f = Self {
            src_tags: HashMap::new(),
            tags: HashMap::new(),
            current_index: 0,
        };
        f.populate_src_tags(store);
        f
    }

    fn populate_src_tags(&mut self, store: &Store) {
        for &root_id in &store.root_ids {
            let root = store.node(root_id);
            for &model_id in &root.children {
                let model = store.node(model_id);
                for &group_id in &model.children {
                    self.populate_src_tags_recurse(store, group_id);
                }
            }
        }
    }

    fn populate_src_tags_recurse(&mut self, store: &Store, group_id: NodeId) {
        let node = store.node(group_id);
        if let Some(ref gi) = node.group_info {
            self.src_tags.insert(gi.name, group_id);
        }
        let children: Vec<NodeId> = node.children.clone();
        for child_id in children {
            self.populate_src_tags_recurse(store, child_id);
        }
    }

    pub fn keep_tag(&mut self, name_id: usize) {
        if self.src_tags.contains_key(&name_id) {
            self.tags.insert(name_id, self.current_index);
        }
        self.current_index += 1;
    }

    pub fn set_keep_from_text(&mut self, text: &str, store: &Store) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let name = if let Some(pos) = line.rfind('\t') {
                &line[pos+1..]
            } else {
                line
            };
            let name_id = store.strings.get_id(name);
            if let Some(name_id) = name_id {
                self.keep_tag(name_id);
            }
        }
    }

    pub fn run(&self, store: &Store) -> Store {
        let mut dst = Store::new();

        // Copy string table
        dst.strings = store.strings.clone();

        let mut group_ids: HashMap<NodeId, i32> = HashMap::new();
        for &root_id in &store.root_ids {
            let root = store.node(root_id);
            for &model_id in &root.children {
                let model = store.node(model_id);
                for &group_id in &model.children {
                    self.tag_recurse(store, group_id, -1, &mut group_ids);
                }
            }
        }

        for &root_id in &store.root_ids {
            let dst_root = dst.clone_node(None, root_id);
            let root = store.node(root_id);
            for &model_id in &root.children {
                let dst_model = dst.clone_node(Some(dst_root), model_id);
                let model = store.node(model_id);
                for &group_id in &model.children {
                    self.build_pruned_copy(store, &mut dst, Some(dst_model), group_id, &group_ids, 0);
                }
            }
        }

        dst.update_counts();
        dst
    }

    fn tag_recurse(&self, store: &Store, group_id: NodeId, parent_id: i32, group_ids: &mut HashMap<NodeId, i32>) {
        let node = store.node(group_id);
        let my_id = if let Some(ref gi) = node.group_info {
            if let Some(&tag_id) = self.tags.get(&gi.name) {
                tag_id
            } else {
                parent_id
            }
        } else {
            parent_id
        };

        group_ids.insert(group_id, my_id);
        let children: Vec<NodeId> = node.children.clone();
        for child_id in children {
            self.tag_recurse(store, child_id, my_id, group_ids);
        }
    }

    fn build_pruned_copy(
        &self,
        src: &Store,
        dst: &mut Store,
        dst_parent: Option<NodeId>,
        src_group: NodeId,
        group_ids: &HashMap<NodeId, i32>,
        level: usize,
    ) {
        let my_id = group_ids.get(&src_group).copied().unwrap_or(-1);

        let actual_parent = if my_id == -1 && level < 2 {
            let new_node = dst.clone_node(dst_parent, src_group);
            Some(new_node)
        } else if my_id != -1 {
            let new_node = dst.clone_node(dst_parent, src_group);
            Some(new_node)
        } else {
            dst_parent
        };

        let node = src.node(src_group);
        let geo_ids: Vec<GeometryId> = node.geometry_ids.clone();
        if let Some(parent) = actual_parent {
            for geo_id in geo_ids {
                dst.clone_geometry(parent, geo_id);
            }
        }

        let children: Vec<NodeId> = node.children.clone();
        for child_id in children {
            self.build_pruned_copy(src, dst, actual_parent, child_id, group_ids, level + 1);
        }
    }
}

// Extend StringInterner with lookup by string
impl crate::store::StringInterner {
    pub fn get_id(&self, s: &str) -> Option<usize> {
        self.map.get(s).copied()
    }

    pub fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            strings: self.strings.clone(),
        }
    }
}
