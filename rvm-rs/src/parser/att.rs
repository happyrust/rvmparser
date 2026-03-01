use crate::store::Store;
use crate::store::node::{NodeId, Attribute};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AttError {
    #[error("Line {line}: {msg}")]
    ParseError { line: usize, msg: String },
}

struct StackItem {
    _id: usize,
    group: Option<NodeId>,
}

pub fn parse_att(data: &[u8], store: &mut Store) -> Result<(), AttError> {
    let text = String::from_utf8_lossy(data);
    let header_info_id = store.strings.intern("Header Information");

    let mut stack: Vec<StackItem> = Vec::new();
    let mut lines = text.lines();

    // Skip first line
    lines.next();

    for (line_num, line) in lines.enumerate() {
        let line_num = line_num + 2;
        let trimmed = line.trim_start();

        if trimmed.starts_with("NEW ") || trimmed.starts_with("NEW\t") {
            let id_str = trimmed[4..].trim();
            let id = store.strings.intern(id_str);

            let group = if stack.is_empty() {
                if id != header_info_id {
                    find_root_group(store, id)
                } else {
                    None
                }
            } else {
                let parent_group = stack.last().and_then(|s| s.group);
                if let Some(parent_id) = parent_group {
                    find_child_group(store, parent_id, id)
                } else {
                    None
                }
            };

            stack.push(StackItem { _id: id, group });
        } else if trimmed.starts_with("END") {
            if stack.is_empty() {
                return Err(AttError::ParseError {
                    line: line_num,
                    msg: "More END-tags than NEW-tags".into(),
                });
            }
            stack.pop();
        } else if !trimmed.is_empty() {
            parse_attributes(trimmed, line_num, store, &stack)?;
        }
    }

    if !stack.is_empty() {
        return Err(AttError::ParseError {
            line: 0,
            msg: "More NEW-tags than END-tags".into(),
        });
    }

    store.update_counts();
    Ok(())
}

fn parse_attributes(
    line: &str,
    line_num: usize,
    store: &mut Store,
    stack: &[StackItem],
) -> Result<(), AttError> {
    let group = match stack.last().and_then(|s| s.group) {
        Some(g) => g,
        None => return Ok(()),
    };

    let mut remaining = line;
    loop {
        let assign_pos = remaining.find(":=");
        if assign_pos.is_none() {
            return Err(AttError::ParseError {
                line: line_num,
                msg: "Failed to find ':=' token".into(),
            });
        }
        let assign_pos = assign_pos.unwrap();

        let key = remaining[..assign_pos].trim();
        let after_assign = &remaining[assign_pos + 2..];
        let after_assign = after_assign.trim_start();

        let (value_str, rest) = if let Some(sep_pos) = after_assign.find("&end&") {
            (&after_assign[..sep_pos], Some(&after_assign[sep_pos + 5..]))
        } else {
            (after_assign, None)
        };

        let mut value = value_str.trim();
        if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
            value = &value[1..value.len() - 1];
        }

        let key_id = store.strings.intern(key);
        let val_id = store.strings.intern(value);

        let node = store.node_mut(group);
        if let Some(attr) = node.attributes.iter_mut().find(|a| a.key == key_id) {
            attr.val = val_id;
        } else {
            node.attributes.push(Attribute { key: key_id, val: val_id });
        }

        if let Some(r) = rest {
            remaining = r.trim_start();
        } else {
            break;
        }
    }

    Ok(())
}

fn find_root_group(store: &Store, name_id: usize) -> Option<NodeId> {
    for &root_id in &store.root_ids {
        let root = store.node(root_id);
        for &model_id in &root.children {
            let model = store.node(model_id);
            for &group_id in &model.children {
                let group = store.node(group_id);
                if let Some(ref gi) = group.group_info {
                    if gi.name == name_id {
                        return Some(group_id);
                    }
                }
            }
        }
    }
    None
}

fn find_child_group(store: &Store, parent: NodeId, name_id: usize) -> Option<NodeId> {
    let parent_node = store.node(parent);
    for &child_id in &parent_node.children {
        let child = store.node(child_id);
        if let Some(ref gi) = child.group_info {
            if gi.name == name_id {
                return Some(child_id);
            }
        }
    }
    None
}
