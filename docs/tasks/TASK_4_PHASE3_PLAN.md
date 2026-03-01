# 📋 任务 4 阶段 3 计划：Tessellator 集成

## 目标
将连接检测集成到细分流程中，实现智能端盖生成。

---

## 架构设计

### 方案 A：修改 Tessellate trait（不推荐）
```rust
trait Tessellate {
    fn tessellate(&self, tolerance: f32, scale: f32, connections: &[Connection]) -> Triangulation;
}
```

**缺点**：
- 需要修改所有实现
- 破坏现有 API
- 增加复杂度

### 方案 B：创建高层包装函数（推荐）✅
```rust
// 保持原有 trait 不变
trait Tessellate {
    fn tessellate(&self, tolerance: f32, scale: f32) -> Triangulation;
}

// 新增带连接检测的函数
pub fn tessellate_with_connections(
    geometry: &Geometry,
    tolerance: f32,
    scale: f32,
    connections: &[Connection],
) -> Triangulation {
    // 1. 检查哪些端盖需要生成
    let cap_flags = check_caps(geometry, connections);
    
    // 2. 调用原有细分函数
    let mut tri = match &geometry.kind {
        GeometryKind::Cylinder(cyl) => cyl.tessellate(tolerance, scale),
        // ...
    };
    
    // 3. 根据 cap_flags 移除不需要的端盖
    if !cap_flags[0] || !cap_flags[1] {
        tri = remove_caps(tri, cap_flags);
    }
    
    tri
}
```

**优点**：
- 不破坏现有 API
- 向后兼容
- 清晰的职责分离

---

## 实现步骤

### 步骤 1：端盖标记系统

为每个几何体定义端盖信息：

```rust
pub struct CapInfo {
    /// 端盖数量（0, 1, 或 2）
    pub count: usize,
    /// 每个端盖的顶点范围
    pub vertex_ranges: Vec<Range<usize>>,
    /// 每个端盖的索引范围
    pub index_ranges: Vec<Range<usize>>,
}

impl Cylinder {
    fn cap_info(&self, segments: usize) -> CapInfo {
        let shell_verts = segments * 2;
        CapInfo {
            count: 2,
            vertex_ranges: vec![
                shell_verts..(shell_verts + segments),           // 底部端盖
                (shell_verts + segments)..(shell_verts + segments * 2), // 顶部端盖
            ],
            index_ranges: vec![
                // 计算索引范围...
            ],
        }
    }
}
```

### 步骤 2：连接检查函数

```rust
/// 检查几何体的哪些端盖需要生成
pub fn check_caps(geometry: &Geometry, connections: &[Connection]) -> Vec<bool> {
    let cap_count = get_cap_count(&geometry.kind);
    let mut generate_caps = vec![true; cap_count];
    
    for (i, generate) in generate_caps.iter_mut().enumerate() {
        // 查找涉及这个端盖的连接
        if let Some(conn) = find_connection_for_cap(geometry, i, connections) {
            // 提取两个接口
            let iface1 = get_interface(geometry, i);
            let iface2 = get_other_interface(geometry, conn, i);
            
            // 检查是否匹配
            if interfaces_match(&iface1, &iface2) {
                *generate = false;
            }
        }
    }
    
    generate_caps
}

fn get_cap_count(kind: &GeometryKind) -> usize {
    match kind {
        GeometryKind::Cylinder(_) => 2,
        GeometryKind::Snout(_) => 2,
        GeometryKind::CircularTorus(_) => 2,
        GeometryKind::RectangularTorus(_) => 2,
        GeometryKind::EllipticalDish(_) => 1,
        GeometryKind::SphericalDish(_) => 1,
        GeometryKind::Pyramid(_) => 6,  // 4 侧面 + 2 端面
        GeometryKind::Box(_) => 6,
        _ => 0,
    }
}
```

### 步骤 3：端盖移除函数

```rust
/// 从三角化结果中移除指定的端盖
pub fn remove_caps(mut tri: Triangulation, generate_caps: &[bool]) -> Triangulation {
    // 这个实现比较复杂，需要：
    // 1. 识别哪些顶点属于端盖
    // 2. 识别哪些三角形使用了这些顶点
    // 3. 移除这些顶点和三角形
    // 4. 重新索引剩余的三角形
    
    // 简化方案：在细分时就不生成端盖
    // 这需要修改每个几何体的 tessellate 实现
    tri
}
```

---

## 简化方案：条件细分（推荐）

更好的方案是在细分时就根据连接信息决定是否生成端盖：

```rust
// 为 Tessellate trait 添加可选的连接参数
trait TessellateWithCaps {
    fn tessellate_with_caps(
        &self,
        tolerance: f32,
        scale: f32,
        generate_caps: &[bool],
    ) -> Triangulation;
}

impl TessellateWithCaps for Cylinder {
    fn tessellate_with_caps(
        &self,
        tolerance: f32,
        scale: f32,
        generate_caps: &[bool],
    ) -> Triangulation {
        // ... 生成壳体 ...
        
        // 条件生成底部端盖
        if generate_caps.get(0).copied().unwrap_or(true) {
            // 生成底部端盖
        }
        
        // 条件生成顶部端盖
        if generate_caps.get(1).copied().unwrap_or(true) {
            // 生成顶部端盖
        }
        
        tri
    }
}

// 默认实现：生成所有端盖
impl<T: TessellateWithCaps> Tessellate for T {
    fn tessellate(&self, tolerance: f32, scale: f32) -> Triangulation {
        self.tessellate_with_caps(tolerance, scale, &[true, true])
    }
}
```

---

## C++ 参考实现

### 连接检查
```cpp
bool cap[2] = { true, true };
for (unsigned i = 0; i < 2; i++) {
    auto * con = geo->connections[i];
    if (con && con->flags == Connection::Flags::HasCircularSide) {
        if (doInterfacesMatch(geo, con)) {
            cap[i] = false;
            discardedCaps++;
        }
    }
}
```

### 条件细分
```cpp
// 计算顶点数
tri->vertices_n = (shell ? 2 * samples : 0) 
                + (cap[0] ? samples : 0) 
                + (cap[1] ? samples : 0);

// 生成底部端盖
if (cap[0]) {
    for (unsigned i = 0; i < samples; i++) {
        // 添加端盖顶点
    }
}

// 生成顶部端盖
if (cap[1]) {
    for (unsigned i = 0; i < samples; i++) {
        // 添加端盖顶点
    }
}
```

---

## 实现优先级

### 高优先级（必须实现）
1. ✅ Cylinder - 最常见的几何体
2. ✅ Snout - 常见且有两个不同半径的端盖
3. ✅ CircularTorus - 管道连接

### 中优先级
4. ⏳ RectangularTorus - 矩形管道
5. ⏳ EllipticalDish - 碟形端盖
6. ⏳ SphericalDish - 球形端盖

### 低优先级
7. ⏳ Pyramid - 复杂（6 个面）
8. ⏳ Box - 复杂（6 个面）

---

## 测试策略

### 单元测试
```rust
#[test]
fn test_cylinder_with_no_caps() {
    let cylinder = Cylinder { radius: 1.0, height: 2.0 };
    let tri = cylinder.tessellate_with_caps(0.01, 1.0, &[false, false]);
    
    // 验证没有端盖顶点
    // 只有壳体顶点
}

#[test]
fn test_cylinder_with_one_cap() {
    let cylinder = Cylinder { radius: 1.0, height: 2.0 };
    let tri = cylinder.tessellate_with_caps(0.01, 1.0, &[true, false]);
    
    // 验证只有底部端盖
}
```

### 集成测试
```rust
#[test]
fn test_connected_cylinders() {
    // 创建两个连接的圆柱
    let cyl1 = create_cylinder(...);
    let cyl2 = create_cylinder(...);
    
    // 创建连接
    let conn = Connection::new(...);
    
    // 细分
    let tri1 = tessellate_with_connections(&cyl1, 0.01, 1.0, &[conn]);
    let tri2 = tessellate_with_connections(&cyl2, 0.01, 1.0, &[conn]);
    
    // 验证端盖被移除
    assert!(tri1.vertices.len() < expected_with_caps);
}
```

---

## 性能考虑

### 预期提升
- **顶点数减少**: 10-20%（取决于连接密度）
- **三角形数减少**: 20-40%（端盖通常占很大比例）
- **文件大小减少**: 20-40%

### 性能开销
- 连接检查：O(n) 其中 n 是连接数
- 接口匹配：O(1) 对于圆形，O(4) 对于矩形
- 总开销：可忽略（< 1% 细分时间）

---

## 下一步行动

### 立即开始
1. 实现 `TessellateWithCaps` trait
2. 为 Cylinder 实现条件端盖生成
3. 添加单元测试

### 后续工作
4. 为其他几何体实现条件端盖生成
5. 实现 `tessellate_with_connections` 包装函数
6. 集成测试和性能测试

---

## 风险和挑战

### 技术风险
- **复杂度**: 每个几何体的端盖生成逻辑不同
- **测试**: 需要大量测试用例验证正确性
- **性能**: 需要确保没有引入性能回归

### 缓解措施
- 从简单几何体开始（Cylinder）
- 充分的单元测试
- 性能基准测试

---

**文档版本**: 1.0  
**创建时间**: 2024  
**作者**: Kiro AI
