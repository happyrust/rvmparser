# Design Document

## Overview

本设计文档描述了 RVM 层次结构处理工具的架构和实现方案，包括三个主要功能：正则表达式扁平化、组保留/合并、和组丢弃。这些工具通过简化场景图结构，提高后续处理的效率。

设计原则：
1. **不可变性**：保留操作创建新 Store，不修改原始数据
2. **模块化**：每个操作独立实现
3. **灵活性**：支持多种过滤和处理方式
4. **性能**：使用高效的数据结构和算法

## Architecture

### 模块结构

```
rvm-rs/
├── src/
│   ├── hierarchy/
│   │   ├── mod.rs           # 层次工具模块入口
│   │   ├── flatten_regex.rs # 正则表达式扁平化
│   │   ├── flatten_keep.rs  # 保留组扁平化
│   │   └── discard.rs       # 丢弃组
│   ├── store/
│   │   └── mod.rs           # Store（需添加克隆方法）
│   └── main.rs              # 命令行入口（需更新）
```

### 数据流

1. **正则扁平化**: 遍历场景图 → 检查名称匹配 → 移动或保留节点
2. **保留操作**: 读取标签列表 → 标记保留节点 → 创建新 Store → 复制保留节点
3. **丢弃操作**: 读取标签列表 → 标记丢弃节点 → 从场景图移除

## Components and Interfaces

### 1. 正则表达式扁平化 (`hierarchy/flatten_regex.rs`)

```rust
use regex::Regex;

pub struct RegexFlattener {
    regex: Regex,
}

pub struct FlattenResult {
    pub nodes_before: usize,
    pub nodes_after: usize,
    pub geometries_before: usize,
    pub geometries_after: usize,
}

impl RegexFlattener {
    pub fn new(pattern: &str) -> Result<Self, HierarchyError>;
    
    pub fn flatten(&self, store: &mut Store) -> FlattenResult;
    
    fn should_keep(&self, name: &str) -> bool;
    
    fn process_children(
        &self,
        nearest_kept_ancestor: NodeId,
        parent: NodeId,
        store: &mut Store,
    );
    
    fn move_geometries(
        &self,
        from: NodeId,
        to: NodeId,
        store: &mut Store,
    );
    
    fn move_attributes(
        &self,
        from: NodeId,
        to: NodeId,
        store: &mut Store,
    );
}
```

### 2. 保留组扁平化 (`hierarchy/flatten_keep.rs`)

```rust
use std::collections::{HashMap, HashSet};

pub struct KeepFlattener {
    keep_tags: HashSet<String>,
    tag_index: HashMap<String, u32>,
}

pub struct KeepResult {
    pub selected_tags: usize,
    pub active_tags: usize,
    pub nodes_in_new_store: usize,
}

impl KeepFlattener {
    pub fn new() -> Self;
    
    pub fn add_keep_tag(&mut self, tag: &str);
    
    pub fn load_keep_file(&mut self, path: &str) -> Result<(), HierarchyError>;
    
    pub fn flatten(&self, source_store: &Store) -> Result<Store, HierarchyError>;
    
    fn populate_source_tags(&self, store: &Store) -> HashMap<String, NodeId>;
    
    fn mark_selected_and_ancestors(
        &self,
        node: NodeId,
        store: &Store,
        tags: &HashMap<String, NodeId>,
    ) -> bool;
    
    fn build_pruned_copy(
        &self,
        source_store: &Store,
        dest_store: &mut Store,
        source_node: NodeId,
        dest_parent: NodeId,
        level: usize,
    );
}
```

### 3. 丢弃组 (`hierarchy/discard.rs`)

```rust
use std::collections::HashSet;

pub struct GroupDiscarder {
    discard_tags: HashSet<String>,
}

pub struct DiscardResult {
    pub discarded_count: usize,
}

impl GroupDiscarder {
    pub fn new() -> Self;
    
    pub fn add_discard_tag(&mut self, tag: &str);
    
    pub fn load_discard_file(&mut self, path: &str) -> Result<(), HierarchyError>;
    
    pub fn discard(&self, store: &mut Store) -> DiscardResult;
    
    fn prune_children(&self, node: NodeId, store: &mut Store) -> usize;
    
    fn should_discard(&self, name: &str) -> bool;
}
```

### 4. Store 扩展 (`store/mod.rs`)

需要为 Store 添加克隆方法：

```rust
impl Store {
    pub fn clone_node(
        &mut self,
        parent: Option<NodeId>,
        source: &Node,
    ) -> NodeId;
    
    pub fn clone_geometry(
        &mut self,
        parent: NodeId,
        source: &Geometry,
    ) -> GeometryId;
    
    pub fn remove_node(&mut self, node: NodeId);
    
    pub fn move_geometries(
        &mut self,
        from: NodeId,
        to: NodeId,
    );
    
    pub fn move_attributes(
        &mut self,
        from: NodeId,
        to: NodeId,
    );
}
```

### 5. 错误类型 (`hierarchy/mod.rs`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum HierarchyError {
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid tag file format: {0}")]
    InvalidTagFile(String),
    
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),
    
    #[error("Circular reference detected")]
    CircularReference,
}
```

## Data Models

### 标签文件格式

```
# 注释行（可选）
/SITE/AREA1/EQUIPMENT1
/SITE/AREA1/EQUIPMENT2
    /SITE/AREA2/PIPE-001  # 缩进会被忽略
/SITE/AREA3/VALVE-042

# 空行会被跳过
```

### 节点标记

在保留操作中，使用临时标记：

```rust
// 在 Node 中添加临时字段（或使用外部 HashMap）
struct NodeMark {
    selected: bool,      // 是否在保留列表中
    has_selected_child: bool,  // 是否有子节点在列表中
    tag_index: Option<u32>,    // 标签索引
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Regex compilation

*For any* valid regex pattern string, the system should successfully compile it or return a clear error
**Validates: Requirements 1.1, 1.5**

### Property 2: Geometry preservation

*For any* flattened scene graph, all geometries from the original graph should be present in the result
**Validates: Requirements 4.1, 4.5**

### Property 3: Attribute preservation

*For any* flattened scene graph, all attributes from the original graph should be present in the result
**Validates: Requirements 4.2, 4.4**

### Property 4: Hierarchy consistency

*For any* node in the flattened graph, its parent should be a kept node or null (for roots)
**Validates: Requirements 5.4**

### Property 5: Keep tag matching

*For any* node whose name is in the keep list, that node should be present in the new Store
**Validates: Requirements 2.2**

### Property 6: Discard completeness

*For any* node whose name is in the discard list, that node and all its descendants should be removed
**Validates: Requirements 3.2**

### Property 7: Minimum level preservation

*For any* flattened graph, at least the first two levels (File, Model, first Group) should be preserved
**Validates: Requirements 8.1, 8.2**

### Property 8: Tag file parsing

*For any* valid tag file with one tag per line, all non-empty lines should be parsed as tags
**Validates: Requirements 7.1, 7.4**

### Property 9: Source Store immutability (for keep operation)

*For any* keep operation, the source Store should remain unchanged after the operation
**Validates: Requirements 6.5**

### Property 10: Node count reduction

*For any* flatten operation, the number of nodes in the result should be less than or equal to the original
**Validates: Requirements 1.4**

## Error Handling

### 错误处理策略

```rust
// 正则表达式错误
match Regex::new(pattern) {
    Ok(regex) => { /* 使用 regex */ },
    Err(e) => return Err(HierarchyError::InvalidRegex(e)),
}

// 文件读取错误
let content = std::fs::read_to_string(path)
    .map_err(HierarchyError::Io)?;

// 节点不存在错误
let node = store.get_node(node_id)
    .ok_or(HierarchyError::NodeNotFound(node_id))?;
```

## Testing Strategy

### 单元测试

**正则扁平化测试**:
- 测试简单的匹配模式
- 测试复杂的正则表达式
- 测试几何体和属性移动
- 测试最低层级保留

**保留操作测试**:
- 测试单个标签保留
- 测试多个标签保留
- 测试祖先路径保留
- 测试新 Store 创建

**丢弃操作测试**:
- 测试单个组丢弃
- 测试递归丢弃
- 测试丢弃计数

### 集成测试

**端到端测试**:
- 解析 RVM 文件
- 应用层次工具
- 验证结果结构
- 导出并验证

### 性能测试

- 测试大型场景图的处理时间
- 测试内存使用
- 测试正则表达式性能

## Implementation Notes

### 正则扁平化实现

```rust
fn process_children(
    &self,
    nearest_kept_ancestor: NodeId,
    parent: NodeId,
    store: &mut Store,
) {
    // 获取并清空父节点的子节点列表
    let children = store.take_children(parent);
    
    for child in children {
        let child_node = store.get_node(child).unwrap();
        
        // 检查是否应该保留
        if self.should_keep(child_node.name()) {
            // 保留：添加到最近保留祖先
            store.add_child(nearest_kept_ancestor, child);
            
            // 递归处理，使用当前节点作为最近保留祖先
            self.process_children(child, child, store);
        } else {
            // 丢弃：移动几何体和属性
            self.move_geometries(child, nearest_kept_ancestor, store);
            self.move_attributes(child, nearest_kept_ancestor, store);
            
            // 递归处理，使用相同的最近保留祖先
            self.process_children(nearest_kept_ancestor, child, store);
        }
    }
}
```

### 保留操作实现

```rust
fn build_pruned_copy(
    &self,
    source_store: &Store,
    dest_store: &mut Store,
    source_node: NodeId,
    dest_parent: NodeId,
    level: usize,
) {
    let source = source_store.get_node(source_node).unwrap();
    
    // 检查是否应该保留
    let should_keep = source.is_marked_for_keep() || level < 2;
    
    let dest_node = if should_keep {
        // 创建新节点
        dest_store.clone_node(Some(dest_parent), source)
    } else {
        // 使用父节点
        dest_parent
    };
    
    // 复制几何体
    for geo_id in source.geometries() {
        let geo = source_store.get_geometry(geo_id).unwrap();
        dest_store.clone_geometry(dest_node, geo);
    }
    
    // 递归处理子节点
    for child_id in source.children() {
        self.build_pruned_copy(
            source_store,
            dest_store,
            child_id,
            dest_node,
            level + 1,
        );
    }
}
```

### 标签文件解析

```rust
fn parse_tag_file(path: &str) -> Result<Vec<String>, HierarchyError> {
    let content = std::fs::read_to_string(path)?;
    
    let tags: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();
    
    Ok(tags)
}
```

## Dependencies

新增依赖：

```toml
[dependencies]
regex = "1.10"
```

## Build and Development

### 开发流程

1. 添加 regex 依赖
2. 实现正则扁平化
3. 实现保留操作
4. 实现丢弃操作
5. 扩展 Store 的克隆方法
6. 集成到主程序
7. 添加测试
8. 性能优化

### 测试命令

```bash
# 单元测试
cargo test hierarchy::

# 集成测试
cargo test --test hierarchy_test

# 测试正则扁平化
cargo run -- input.rvm --keep-regex "^/SITE/.*"

# 测试保留操作
cargo run -- input.rvm --keep-groups keep_list.txt

# 测试丢弃操作
cargo run -- input.rvm --discard-groups discard_list.txt
```

### 使用示例

```bash
# 只保留以 /SITE/ 开头的组
cargo run -- model.rvm --keep-regex "^/SITE/.*" --export-obj output.obj

# 保留指定的组列表
cargo run -- model.rvm --keep-groups important_groups.txt --export-gltf output.gltf

# 丢弃临时组
cargo run -- model.rvm --discard-groups temp_groups.txt --export-json output.json

# 组合使用
cargo run -- model.rvm \
    --discard-groups temp.txt \
    --keep-regex "^/SITE/AREA[12]/.*" \
    --export-obj output.obj
```
