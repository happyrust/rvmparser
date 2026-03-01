use std::io::Write;
use serde_json::{json, Value};
use crate::store::Store;
use crate::store::node::{NodeId, NodeKind};
use super::ExportError;

pub fn export_json(store: &Store, path: &str) -> Result<(), ExportError> {
    let mut roots = Vec::new();
    for &root_id in &store.root_ids {
        roots.push(build_node_json(store, root_id));
    }

    let doc = json!({ "roots": roots });
    let mut file = std::fs::File::create(path)?;
    let formatted = serde_json::to_string_pretty(&doc)
        .map_err(|e| ExportError::Other(e.to_string()))?;
    file.write_all(formatted.as_bytes())?;
    Ok(())
}

fn build_node_json(store: &Store, node_id: NodeId) -> Value {
    let node = store.node(node_id);
    let mut obj = serde_json::Map::new();

    match node.kind {
        NodeKind::File => {
            obj.insert("type".into(), json!("file"));
            if let Some(ref fi) = node.file_info {
                obj.insert("info".into(), json!(store.strings.get(fi.info)));
                obj.insert("note".into(), json!(store.strings.get(fi.note)));
                obj.insert("date".into(), json!(store.strings.get(fi.date)));
                obj.insert("user".into(), json!(store.strings.get(fi.user)));
                obj.insert("encoding".into(), json!(store.strings.get(fi.encoding)));
            }
        }
        NodeKind::Model => {
            obj.insert("type".into(), json!("model"));
            if let Some(ref mi) = node.model_info {
                obj.insert("project".into(), json!(store.strings.get(mi.project)));
                obj.insert("name".into(), json!(store.strings.get(mi.name)));
            }
        }
        NodeKind::Group => {
            obj.insert("type".into(), json!("group"));
            if let Some(ref gi) = node.group_info {
                obj.insert("name".into(), json!(store.strings.get(gi.name)));
                obj.insert("material".into(), json!(gi.material));
                if gi.bbox_world.is_not_empty() {
                    obj.insert("bbox".into(), json!({
                        "min": [gi.bbox_world.min.x, gi.bbox_world.min.y, gi.bbox_world.min.z],
                        "max": [gi.bbox_world.max.x, gi.bbox_world.max.y, gi.bbox_world.max.z],
                    }));
                }
            }
            if !node.attributes.is_empty() {
                let mut attrs = serde_json::Map::new();
                for attr in &node.attributes {
                    let key = store.strings.get(attr.key).to_string();
                    let val = store.strings.get(attr.val).to_string();
                    attrs.insert(key, json!(val));
                }
                obj.insert("attributes".into(), Value::Object(attrs));
            }
        }
    }

    let children: Vec<Value> = node.children.iter()
        .map(|&cid| build_node_json(store, cid))
        .collect();

    if !children.is_empty() {
        obj.insert("children".into(), Value::Array(children));
    }

    Value::Object(obj)
}
