pub mod stats;

use crate::store::Store;
use crate::store::node::{NodeId, NodeKind};
use crate::store::geometry::GeometryId;

pub trait StoreVisitor {
    fn init(&mut self, _store: &Store) {}
    fn done(&mut self) {}
    fn begin_file(&mut self, _store: &Store, _node: NodeId) {}
    fn end_file(&mut self) {}
    fn begin_model(&mut self, _store: &Store, _node: NodeId) {}
    fn end_model(&mut self) {}
    fn begin_group(&mut self, _store: &Store, _node: NodeId) {}
    fn end_group(&mut self) {}
    fn attribute(&mut self, _key: &str, _val: &str) {}
    fn geometry(&mut self, _store: &Store, _geo: GeometryId) {}
}

pub fn apply_visitor(store: &Store, visitor: &mut dyn StoreVisitor) {
    visitor.init(store);
    for &root_id in &store.root_ids {
        apply_visitor_node(store, visitor, root_id);
    }
    visitor.done();
}

fn apply_visitor_node(store: &Store, visitor: &mut dyn StoreVisitor, node_id: NodeId) {
    let node = store.node(node_id);
    match node.kind {
        NodeKind::File => {
            visitor.begin_file(store, node_id);
            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                apply_visitor_node(store, visitor, child_id);
            }
            visitor.end_file();
        }
        NodeKind::Model => {
            visitor.begin_model(store, node_id);
            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                apply_visitor_node(store, visitor, child_id);
            }
            visitor.end_model();
        }
        NodeKind::Group => {
            visitor.begin_group(store, node_id);

            for attr in &node.attributes {
                let key = store.strings.get(attr.key);
                let val = store.strings.get(attr.val);
                visitor.attribute(key, val);
            }

            let geo_ids: Vec<GeometryId> = node.geometry_ids.clone();
            for geo_id in geo_ids {
                visitor.geometry(store, geo_id);
            }

            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                apply_visitor_node(store, visitor, child_id);
            }

            visitor.end_group();
        }
    }
}
