# Design Document

## Overview

本设计文档描述了 RVM 几何处理功能的架构和实现方案，包括连接检测（Connection Detection）和对齐（Alignment）两个核心功能。这些功能通过分析几何体的空间关系，改善导出网格的质量。

设计原则：
1. **空间效率**：使用空间排序优化连接检测
2. **模块化**：连接检测和对齐独立实现
3. **可扩展性**：易于添加新几何类型的支持
4. **性能优先**：使用高效的数据结构和算法

## Architecture

### 模块结构

```
rvm-rs/
├── src/
│   ├── processing/
│   │   ├── mod.rs           # 处理模块入口
│   │   ├── connection.rs    # 连接检测
│   │   ├── alignment.rs     # 对齐处理
│   │   └── anchor.rs        # 锚点生成
│   ├── store/
│   │   ├── geometry.rs      # 几何体定义（需扩展）
│   │   └── connection.rs    # 连接数据结构（新增）
│   └── main.rs              # 命令行入口（需更新）
```

### 数据流

1. **锚点生成**: 遍历场景图，为每个几何体生成锚点
2. **空间排序**: 按 X 坐标对锚点排序
3. **连接检测**: 查找距离和方向匹配的锚点对
4. **连通分量**: 识别通过连接关系相连的几何体组
5. **对齐传播**: 沿连接关系传播采样起始角度

## Components and Interfaces

### 1. 锚点数据结构 (`processing/anchor.rs`)

```rust
use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum ConnectionFlags {
    None = 0,
    HasCircularSide = 1 << 0,
    HasRectangularSide = 1 << 1,
}

#[derive(Debug, Clone)]
pub struct Anchor {
    pub geometry_id: GeometryId,
    pub position: Vec3,        // 世界空间位置
    pub normal: Vec3,          // 世界空间法线（归一化）
    pub offset: usize,         // 几何体上的偏移索引（0-5）
    pub flags: ConnectionFlags,
    pub matched: bool,
}

impl Anchor {
    pub fn new(
        geometry_id: GeometryId,
        position: Vec3,
        normal: Vec3,
        offset: usize,
        flags: ConnectionFlags,
    ) -> Self;
    
    pub fn from_geometry(
        geometry: &Geometry,
        store: &Store,
    ) -> Vec<Anchor>;
}
```

### 2. 连接数据结构 (`store/connection.rs`)

```rust
#[derive(Debug, Clone)]
pub struct Connection {
    pub geometries: [GeometryId; 2],
    pub offsets: [usize; 2],
    pub position: Vec3,
    pub normal: Vec3,
    pub flags: ConnectionFlags,
}

impl Connection {
    pub fn new(
        anchor1: &Anchor,
        anchor2: &Anchor,
    ) -> Self;
    
    pub fn has_circular_side(&self) -> bool;
    pub fn has_rectangular_side(&self) -> bool;
}
```

### 3. 连接检测器 (`processing/connection.rs`)

```rust
pub struct ConnectionDetector {
    distance_threshold: f32,
    angle_threshold: f32,
}

pub struct ConnectionResult {
    pub connections: Vec<Connection>,
    pub total_anchors: usize,
    pub matched_anchors: usize,
}

impl ConnectionDetector {
    pub fn new(distance_threshold: f32, angle_threshold: f32) -> Self;
    
    pub fn detect_connections(
        &self,
        store: &Store,
    ) -> ConnectionResult;
    
    fn generate_anchors(&self, store: &Store) -> Vec<Anchor>;
    
    fn sort_anchors(&self, anchors: &mut [Anchor]);
    
    fn find_matches(
        &self,
        anchors: &mut [Anchor],
    ) -> Vec<Connection>;
    
    fn is_match(
        &self,
        anchor1: &Anchor,
        anchor2: &Anchor,
    ) -> bool;
}
```

### 4. 对齐处理器 (`processing/alignment.rs`)

```rust
pub struct AlignmentProcessor {
    connections: Vec<Connection>,
}

pub struct AlignmentResult {
    pub connected_components: usize,
    pub aligned_geometries: usize,
}

impl AlignmentProcessor {
    pub fn new(connections: Vec<Connection>) -> Self;
    
    pub fn align_geometries(
        &mut self,
        store: &mut Store,
    ) -> AlignmentResult;
    
    fn find_connected_components(&self) -> Vec<Vec<GeometryId>>;
    
    fn align_component(
        &self,
        component: &[GeometryId],
        store: &mut Store,
    );
    
    fn align_cylinder(
        &self,
        geometry: &mut Geometry,
        offset: usize,
        up_world: Vec3,
    );
    
    fn align_circular_torus(
        &self,
        geometry: &mut Geometry,
        offset: usize,
        up_world: Vec3,
    );
    
    fn align_snout(
        &self,
        geometry: &mut Geometry,
        offset: usize,
        up_world: Vec3,
    );
}
```

### 5. 几何体扩展 (`store/geometry.rs`)

需要为 Geometry 结构添加字段：

```rust
pub struct Geometry {
    // ... 现有字段 ...
    
    pub sample_start_angle: f32,  // 采样起始角度
    pub connections: [Option<ConnectionId>; 6],  // 最多 6 个连接
}
```

## Data Models

### 锚点生成规则

每种几何类型的锚点生成：

```rust
// 金字塔：6 个锚点（4 个侧面 + 2 个底面）
fn generate_pyramid_anchors(pyramid: &Pyramid, transform: &Affine3A) -> Vec<Anchor>;

// 盒子：6 个锚点（6 个面）
fn generate_box_anchors(box_: &Box, transform: &Affine3A) -> Vec<Anchor>;

// 圆柱：2 个锚点（2 个端面）
fn generate_cylinder_anchors(cylinder: &Cylinder, transform: &Affine3A) -> Vec<Anchor>;

// 圆环：2 个锚点（2 个端面）
fn generate_torus_anchors(torus: &CircularTorus, transform: &Affine3A) -> Vec<Anchor>;

// 矩形环：2 个锚点（2 个端面）
fn generate_rect_torus_anchors(torus: &RectangularTorus, transform: &Affine3A) -> Vec<Anchor>;

// Snout：2 个锚点（2 个端面，考虑剪切）
fn generate_snout_anchors(snout: &Snout, transform: &Affine3A) -> Vec<Anchor>;

// Dish：1 个锚点（底面）
fn generate_dish_anchors(dish: &EllipticalDish, transform: &Affine3A) -> Vec<Anchor>;
```

### 连接匹配条件

```rust
fn is_match(anchor1: &Anchor, anchor2: &Anchor, threshold: f32, angle_threshold: f32) -> bool {
    // 1. 未匹配
    if anchor1.matched || anchor2.matched {
        return false;
    }
    
    // 2. 距离检查
    let distance = (anchor1.position - anchor2.position).length();
    if distance > threshold {
        return false;
    }
    
    // 3. 法线对齐检查（相反方向）
    let dot = anchor1.normal.dot(anchor2.normal);
    if dot > -angle_threshold {  // 通常 angle_threshold = 0.98 (约 11.5 度)
        return false;
    }
    
    true
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Anchor count consistency

*For any* geometry type, the number of generated anchors should match the expected count for that type
**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 2: World space transformation

*For any* anchor, its position and normal should be correctly transformed from local to world space
**Validates: Requirements 7.1, 7.2, 7.3**

### Property 3: Normal vector normalization

*For any* anchor, its normal vector should have length approximately 1.0
**Validates: Requirements 7.5**

### Property 4: Connection symmetry

*For any* connection between geometries A and B, the connection should reference both geometries
**Validates: Requirements 1.5**

### Property 5: Matched anchor uniqueness

*For any* matched anchor, it should appear in exactly one connection
**Validates: Requirements 6.4**

### Property 6: Connection flag consistency

*For any* connection, if either anchor has HasCircularSide flag, the connection should have that flag
**Validates: Requirements 3.3**

### Property 7: Alignment angle validity

*For any* aligned geometry, the sample_start_angle should be in the range [0, 2π)
**Validates: Requirements 4.4**

### Property 8: Connected component coverage

*For any* geometry with connections, it should belong to exactly one connected component
**Validates: Requirements 5.1**

### Property 9: Distance threshold enforcement

*For any* connection, the distance between anchor positions should be less than or equal to the threshold
**Validates: Requirements 6.3**

### Property 10: Alignment propagation

*For any* connected component, all circular geometries should have their sample_start_angle set
**Validates: Requirements 4.5**

## Error Handling

### 处理错误

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Invalid geometry type for anchor generation: {0}")]
    InvalidGeometryType(String),
    
    #[error("Transform matrix is not invertible")]
    NonInvertibleTransform,
    
    #[error("Connection detection failed: {0}")]
    ConnectionDetectionFailed(String),
    
    #[error("Alignment failed: {0}")]
    AlignmentFailed(String),
}
```

### 错误处理策略

- 无效几何类型跳过并记录警告
- 变换矩阵问题使用单位矩阵
- 连接检测失败不影响后续处理
- 对齐失败保持默认角度

## Testing Strategy

### 单元测试

**锚点生成测试**:
- 测试每种几何类型的锚点数量
- 测试锚点位置和法线的正确性
- 测试变换矩阵的应用
- 测试边界情况

**连接检测测试**:
- 测试简单的两几何体连接
- 测试距离阈值
- 测试法线对齐检查
- 测试空间排序

**对齐测试**:
- 测试圆柱对齐
- 测试圆环对齐
- 测试连通分量识别
- 测试角度传播

### 集成测试

**端到端测试**:
- 解析包含连接几何体的 RVM 文件
- 执行连接检测
- 执行对齐
- 验证结果

### 性能测试

- 测试大量几何体的连接检测时间
- 测试空间排序的效率
- 测试内存使用

## Implementation Notes

### 空间排序优化

```rust
// 按 X 坐标排序锚点
anchors.sort_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap());

// 搜索时利用排序
for j in 0..anchors.len() {
    if anchors[j].matched {
        continue;
    }
    
    for i in (j+1)..anchors.len() {
        // 如果 X 坐标差距过大，后续锚点都不可能匹配
        if anchors[i].position.x > anchors[j].position.x + threshold {
            break;
        }
        
        if is_match(&anchors[j], &anchors[i], threshold, angle_threshold) {
            // 创建连接
            create_connection(&anchors[j], &anchors[i]);
            break;
        }
    }
}
```

### 对齐角度计算

```rust
// 圆柱对齐
fn align_cylinder(geometry: &mut Geometry, up_world: Vec3) {
    let transform_inv = geometry.transform.inverse();
    let up_local = transform_inv.transform_vector3(up_world);
    
    // 投影到 XY 平面
    let up_local_xy = Vec3::new(up_local.x, up_local.y, 0.0).normalize();
    
    // 计算角度
    geometry.sample_start_angle = up_local_xy.y.atan2(up_local_xy.x);
}

// 圆环对齐
fn align_circular_torus(geometry: &mut Geometry, offset: usize, up_world: Vec3) {
    let torus = &geometry.kind.circular_torus();
    let transform_inv = geometry.transform.inverse();
    let up_local = transform_inv.transform_vector3(up_world);
    
    // 如果是第二个端面，需要旋转回第一个端面
    let up_local = if offset == 1 {
        let c = torus.angle.cos();
        let s = torus.angle.sin();
        Vec3::new(
            c * up_local.x + s * up_local.y,
            -s * up_local.x + c * up_local.y,
            up_local.z,
        )
    } else {
        up_local
    };
    
    geometry.sample_start_angle = up_local.z.atan2(up_local.x);
}
```

### 连通分量识别

使用广度优先搜索：

```rust
fn find_connected_components(connections: &[Connection]) -> Vec<Vec<GeometryId>> {
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    
    // 构建邻接表
    let mut adjacency: HashMap<GeometryId, Vec<(GeometryId, &Connection)>> = HashMap::new();
    for conn in connections {
        adjacency.entry(conn.geometries[0])
            .or_default()
            .push((conn.geometries[1], conn));
        adjacency.entry(conn.geometries[1])
            .or_default()
            .push((conn.geometries[0], conn));
    }
    
    // BFS 遍历
    for &start in adjacency.keys() {
        if visited.contains(&start) {
            continue;
        }
        
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);
        
        while let Some(current) = queue.pop_front() {
            component.push(current);
            
            if let Some(neighbors) = adjacency.get(&current) {
                for &(neighbor, _) in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        components.push(component);
    }
    
    components
}
```

## Dependencies

无需新增依赖，使用现有的：
- `glam` - 向量和矩阵运算
- `thiserror` - 错误处理

## Build and Development

### 开发流程

1. 扩展 Geometry 结构添加连接字段
2. 实现锚点生成
3. 实现连接检测
4. 实现对齐处理
5. 集成到主程序
6. 添加测试
7. 性能优化

### 测试命令

```bash
# 单元测试
cargo test processing::

# 集成测试
cargo test --test geometry_processing_test

# 性能测试
cargo bench --bench connection_bench
```
