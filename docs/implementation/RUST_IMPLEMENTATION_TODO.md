# Rust 实现待办事项分析

## 概述

本文档详细分析 C++ 实现中已有但 Rust 实现中缺失或不完整的功能。

---

## 1. 几何体细分（Tessellation）功能对比

### 1.1 已实现的几何体

| 几何体类型 | C++ | Rust | 完整度 | 备注 |
|-----------|-----|------|--------|------|
| Pyramid | ✅ | ✅ | 🟡 60% | 缺少连接检测和端盖优化 |
| Box | ✅ | ✅ | 🟡 60% | 缺少连接检测和端盖优化 |
| RectangularTorus | ✅ | ✅ | 🟢 95% | 刚补齐，需测试 |
| CircularTorus | ✅ | ✅ | 🟢 95% | 刚补齐，需测试 |
| Cylinder | ✅ | ✅ | 🟡 70% | 缺少连接检测 |
| Sphere | ✅ | ✅ | 🟡 70% | 实现简化 |
| EllipticalDish | ✅ | ✅ | 🟡 60% | 实现简化 |
| SphericalDish | ✅ | ✅ | 🟡 60% | 实现简化 |
| Snout | ✅ | ✅ | 🔴 40% | **缺少 shear 参数支持** |
| Line | ✅ | ✅ | 🟢 80% | 基本完整 |
| FacetGroup | ✅ | ✅ | 🟡 70% | 缺少复杂多边形处理 |

---

## 2. 关键缺失功能

### 2.1 ⚠️ **高优先级：Snout 的 Shear 参数**

#### 问题描述

C++ 中 Snout 有 **6 个剪切参数**，Rust 中标记为 `unknown1-4`：

**C++ 定义**：
```cpp
struct {
    float offset[2];      // X, Y 偏移
    float bshear[2];      // 底部剪切角度 [X方向, Y方向]
    float tshear[2];      // 顶部剪切角度 [X方向, Y方向]
    float radius_b;       // 底部半径
    float radius_t;       // 顶部半径
    float height;         // 高度
} snout;
```

**Rust 当前定义**：
```rust
pub struct Snout {
    pub radius_bottom: f32,
    pub radius_top: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub unknown1: f32,    // 应该是 bshear[0]
    pub unknown2: f32,    // 应该是 bshear[1]
    pub unknown3: f32,    // 应该是 tshear[0]
    pub unknown4: f32,    // 应该是 tshear[1]
}
```

#### Shear 参数的作用

Shear（剪切）使得 Snout 的端面不再垂直于高度方向，而是倾斜的：

```
无剪切:              有剪切:
  ┌─────┐              ╱─────╲
  │     │             ╱       ╲
  │     │            │         │
  │     │            │         │
  └─────┘             ╲       ╱
                       ╲─────╱
```

#### C++ 实现细节

```cpp
float h2 = 0.5f * sn.height;
float ox = 0.5f * sn.offset[0];
float oy = 0.5f * sn.offset[1];

// 将剪切角度转换为斜率
float mb[2] = { std::tan(sn.bshear[0]), std::tan(sn.bshear[1]) };
float mt[2] = { std::tan(sn.tshear[0]), std::tan(sn.tshear[1]) };

// 底部顶点位置（带剪切）
float xb = t1[2*i+0] - ox;
float yb = t1[2*i+1] - oy;
float zb = -h2 + mb[0] * t1[2*i+0] + mb[1] * t1[2*i+1];  // Z 受剪切影响

// 顶部顶点位置（带剪切）
float xt = t2[2*i+0] + ox;
float yt = t2[2*i+1] + oy;
float zt = h2 + mt[0] * t2[2*i+0] + mt[1] * t2[2*i+1];   // Z 受剪切影响

// 法线计算（考虑剪切）
float s = (sn.offset[0] * t0[2*i+0] + sn.offset[1] * t0[2*i+1]);
float nx = t0[2*i+0];
float ny = t0[2*i+1];
float nz = -(sn.radius_t - sn.radius_b + s) / sn.height;

// 端盖法线（考虑剪切）
// 底部端盖
auto nx = std::sin(sn.bshear[0]) * std::cos(sn.bshear[1]);
auto ny = std::sin(sn.bshear[1]);
auto nz = -std::cos(sn.bshear[0]) * std::cos(sn.bshear[1]);

// 顶部端盖
auto nx = -std::sin(sn.tshear[0]) * std::cos(sn.tshear[1]);
auto ny = -std::sin(sn.tshear[1]);
auto nz = std::cos(sn.tshear[0]) * std::cos(sn.tshear[1]);
```

#### 需要的修改

1. **更新 Rust 结构体定义**：
```rust
pub struct Snout {
    pub radius_bottom: f32,
    pub radius_top: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub bottom_shear_x: f32,  // bshear[0]
    pub bottom_shear_y: f32,  // bshear[1]
    pub top_shear_x: f32,     // tshear[0]
    pub top_shear_y: f32,     // tshear[1]
}
```

2. **更新 tessellate 实现**：
   - 计算剪切斜率：`tan(shear_angle)`
   - 应用剪切到顶点 Z 坐标
   - 正确计算侧面法线
   - 正确计算端盖法线

---

### 2.2 ⚠️ **高优先级：连接检测与端盖优化**

#### 问题描述

C++ 实现有完整的 **连接检测系统**，可以：
- 检测相邻几何体的接触面
- 避免生成重复的端盖
- 减少 20-40% 的三角形数量

Rust 实现中 **完全缺失** 这个功能。

#### C++ 实现架构

```cpp
// 1. 连接数据结构
struct Connection {
    enum struct Flags : uint8_t {
        None = 0,
        HasCircularSide = 1<<0,
        HasRectangularSide = 1<<1
    };
    
    Connection* next = nullptr;
    Geometry* geo[2] = { nullptr, nullptr };  // 连接的两个几何体
    unsigned offset[2];                        // 连接面的索引
    Vec3f p;                                   // 连接点位置
    Vec3f d;                                   // 连接方向
    Flags flags = Flags::None;
};

// 2. 接口定义
struct Interface {
    enum struct Kind {
        Undefined,
        Square,      // 矩形接口（Pyramid, Box, RectangularTorus）
        Circular     // 圆形接口（Cylinder, CircularTorus, Snout, Dish）
    };
    Kind kind;
    
    union {
        struct { Vec3f p[4]; } square;      // 矩形的4个角点
        struct { float radius; } circular;   // 圆形的半径
    };
};

// 3. 接口提取
Interface getInterface(const Geometry* geo, unsigned offset);

// 4. 接口匹配
bool doInterfacesMatch(const Geometry* geo, const Connection* con) {
    auto thisIFace = getInterface(geo, thisOffset);
    auto thatIFace = getInterface(otherGeo, thatOffset);
    
    if (thisIFace.kind != thatIFace.kind) return false;
    
    if (thisIFace.kind == Interface::Kind::Circular) {
        // 圆形：比较半径（允许5%误差）
        return thisIFace.circular.radius <= 1.05f * thatIFace.circular.radius;
    }
    else {
        // 矩形：比较4个角点位置
        for (unsigned j = 0; j < 4; j++) {
            bool found = false;
            for (unsigned i = 0; i < 4; i++) {
                if (distanceSquared(thisIFace.square.p[j], 
                                   thatIFace.square.p[i]) < 0.001f * 0.001f) {
                    found = true;
                }
            }
            if (!found) return false;
        }
        return true;
    }
}

// 5. 端盖生成决策
bool cap[2] = { true, true };
for (unsigned i = 0; i < 2; i++) {
    auto * con = geo->connections[i];
    if (con && doInterfacesMatch(geo, con)) {
        cap[i] = false;  // 不生成端盖
        discardedCaps++;
    }
}
```

#### 需要的实现

1. **连接数据结构**：
```rust
pub struct Connection {
    pub geometries: [GeometryId; 2],
    pub offsets: [usize; 2],
    pub position: Vec3,
    pub direction: Vec3,
    pub flags: ConnectionFlags,
}

bitflags! {
    pub struct ConnectionFlags: u8 {
        const NONE = 0;
        const HAS_CIRCULAR_SIDE = 1 << 0;
        const HAS_RECTANGULAR_SIDE = 1 << 1;
    }
}
```

2. **接口系统**：
```rust
pub enum Interface {
    Undefined,
    Square { corners: [Vec3; 4] },
    Circular { radius: f32 },
}

fn get_interface(geo: &Geometry, offset: usize) -> Interface;
fn interfaces_match(iface1: &Interface, iface2: &Interface) -> bool;
```

3. **集成到 tessellate**：
   - 在细分前检查连接
   - 根据匹配结果决定是否生成端盖

---

### 2.3 🟡 **中优先级：sphereBasedShape 统一实现**

#### 问题描述

C++ 中 Sphere, EllipticalDish, SphericalDish 都使用 **统一的 `sphereBasedShape` 函数**：

```cpp
Triangulation* sphereBasedShape(Arena* arena, const Geometry* geo, 
                                float radius, float arc, 
                                float shift_z, float scale_z, float scale);

// Sphere
tri = factory->sphereBasedShape(&arena, geo, 
    0.5f * geo->sphere.diameter,  // radius
    π,                             // arc (完整半球)
    0.f,                           // shift_z
    1.f,                           // scale_z
    scale);

// EllipticalDish
tri = factory->sphereBasedShape(&arena, geo,
    geo->ellipticalDish.baseRadius,                    // radius
    π/2,                                               // arc (四分之一球)
    0.f,                                               // shift_z
    geo->ellipticalDish.height / geo->ellipticalDish.baseRadius,  // scale_z
    scale);

// SphericalDish
float r_sphere = (r_circ*r_circ + h*h) / (2.f*h);
float arc = asin(r_circ / r_sphere);
tri = factory->sphereBasedShape(&arena, geo,
    r_sphere,      // radius
    arc,           // arc
    h - r_sphere,  // shift_z
    1.f,           // scale_z
    scale);
```

#### 优点

- **代码复用**：三种几何体共享同一实现
- **一致性**：保证相似几何体的细分质量一致
- **维护性**：只需维护一份代码

#### 需要的实现

```rust
fn sphere_based_shape(
    radius: f32,
    arc: f32,
    shift_z: f32,
    scale_z: f32,
    tolerance: f32,
    scale: f32,
) -> Triangulation {
    // 统一的球面细分逻辑
    // ...
}

impl Tessellate for Sphere {
    fn tessellate(&self, tolerance: f32, scale: f32) -> Triangulation {
        sphere_based_shape(
            self.radius,
            PI,
            0.0,
            1.0,
            tolerance,
            scale,
        )
    }
}

impl Tessellate for EllipticalDish {
    fn tessellate(&self, tolerance: f32, scale: f32) -> Triangulation {
        sphere_based_shape(
            self.base_radius,
            PI / 2.0,
            0.0,
            self.height / self.base_radius,
            tolerance,
            scale,
        )
    }
}

impl Tessellate for SphericalDish {
    fn tessellate(&self, tolerance: f32, scale: f32) -> Triangulation {
        let r_circ = self.base_radius;
        let h = self.height;
        let r_sphere = (r_circ * r_circ + h * h) / (2.0 * h);
        let sinval = (r_circ / r_sphere).clamp(-1.0, 1.0);
        let mut arc = sinval.asin();
        if r_circ < h {
            arc = PI - arc;
        }
        
        sphere_based_shape(
            r_sphere,
            arc,
            h - r_sphere,
            1.0,
            tolerance,
            scale,
        )
    }
}
```

---

### 2.4 🟡 **中优先级：自适应环数细分**

#### 问题描述

C++ 的 `sphereBasedShape` 使用 **自适应环数**，每个环的采样点数不同：

```cpp
unsigned min_rings = 3;
unsigned rings = unsigned(std::max(float(min_rings), 
                         scale_z * samples * arc * (1.f / twopi)));

u0.resize(rings);
for (unsigned r = 0; r < rings; r++) {
    float theta = theta_scale * r;
    t0[2*r+0] = std::cos(theta);
    t0[2*r+1] = std::sin(theta);
    
    // 每个环的采样点数根据半径调整
    u0[r] = unsigned(std::max(3.f, t0[2*r+1] * samples));
}
u0[0] = 1;  // 顶点
if (is_sphere) {
    u0[rings-1] = 1;  // 底部顶点
}
```

**优点**：
- 极点附近采样少，赤道附近采样多
- 避免极点处的三角形退化
- 更均匀的三角形分布

Rust 当前使用 **固定采样**，所有环的采样点数相同。

---

### 2.5 🟢 **低优先级：FacetGroup 的复杂多边形处理**

#### 问题描述

C++ 使用 **libtess2** 库处理复杂多边形：
- 带孔的多边形
- 自相交多边形
- 多轮廓多边形

```cpp
auto tess = tessNewTess(nullptr);
for (unsigned c = 0; c < poly.contours_n; c++) {
    tessAddContour(tess, 3, cont.vertices, 3*sizeof(float), cont.vertices_n);
}
if (tessTesselate(tess, TESS_WINDING_ODD, TESS_POLYGONS, 3, 3, nullptr)) {
    // 处理细分结果
}
tessDeleteTess(tess);
```

Rust 当前使用 **简单扇形三角化**，只能处理凸多边形。

#### 解决方案

可以使用 Rust 的三角化库：
- `earcutr` - Earcut 算法的 Rust 实现
- `lyon_tessellation` - 2D 路径细分
- `spade` - Delaunay 三角化

---

## 3. 性能优化功能

### 3.1 缺失的优化

| 优化功能 | C++ | Rust | 影响 |
|---------|-----|------|------|
| Arena 内存分配器 | ✅ | ❌ | 减少内存碎片，提高分配速度 |
| 几何体缓存 | ✅ | ❌ | 避免重复细分相同几何体 |
| 端盖优化 | ✅ | ❌ | 减少 20-40% 三角形 |
| 小几何体剔除 | ✅ | ❌ | 跳过过小的几何体 |
| 误差累积 | ✅ | ❌ | 层级误差传播 |

### 3.2 Arena 分配器

C++ 使用自定义 Arena 分配器：

```cpp
class Arena {
public:
    void* alloc(size_t size);
    void clear();
    
private:
    std::vector<char*> blocks;
    char* current;
    size_t remaining;
};

// 使用
tri->vertices = (float*)arena->alloc(3 * sizeof(float) * tri->vertices_n);
tri->normals = (float*)arena->alloc(3 * sizeof(float) * tri->vertices_n);
tri->indices = (uint32_t*)arena->alloc(3 * sizeof(uint32_t) * tri->triangles_n);
```

**优点**：
- 批量分配，减少系统调用
- 连续内存，缓存友好
- 统一释放，避免内存泄漏

Rust 可以使用 `bumpalo` 或 `typed-arena` crate。

### 3.3 几何体缓存

C++ 缓存相同几何体的细分结果：

```cpp
struct CacheItem {
    CacheItem* next;
    Geometry* src;
    Triangulation* tri;
};

Triangulation* getTriangulation(Geometry* geo) {
    auto hash = fnv_1a((const char*)geo + offset, size);
    
    // 查找缓存
    for (auto * item = cache[hash]; item != nullptr; item = item->next) {
        if (std::memcmp(geo, item->src, size) == 0) {
            return item->tri;  // 缓存命中
        }
    }
    
    // 缓存未命中，创建新的
    auto * tri = tessellate(geo);
    cache_insert(hash, geo, tri);
    return tri;
}
```

### 3.4 小几何体剔除

C++ 可以跳过过小的几何体：

```cpp
void geometry(Geometry* geo) {
    auto scale = getScale(geo->M_3x4);
    
    // 组级别剔除
    if (stack[stack_p-1].groupError < cullLeafThresholdScaled) {
        geo->triangulation = createEmptyTriangulation();
        geo->triangulation->error = stack[stack_p-1].groupError;
        return;
    }
    
    // 几何体级别剔除
    auto scaledDiagonal = diagonal(geo->bboxWorld);
    if (scaledDiagonal < cullGeometryThresholdScaled) {
        geo->triangulation = createEmptyTriangulation();
        geo->triangulation->error = scaledDiagonal;
        geometryCulled++;
        return;
    }
    
    // 正常细分
    tessellate(geo);
}
```

---

## 4. 数据结构差异

### 4.1 内存布局

**C++**：
```cpp
struct Triangulation {
    float* vertices;      // 裸指针，Arena 分配
    float* normals;       // 裸指针，Arena 分配
    uint32_t* indices;    // 裸指针，Arena 分配
    uint32_t vertices_n;
    uint32_t triangles_n;
    float error;
};
```

**Rust**：
```rust
pub struct Triangulation {
    pub vertices: Vec<f32>,   // 动态数组，堆分配
    pub normals: Vec<f32>,    // 动态数组，堆分配
    pub indices: Vec<u32>,    // 动态数组，堆分配
    pub error: f32,
}
```

**影响**：
- Rust 的 Vec 更安全但可能有额外开销
- C++ 的裸指针需要手动管理但更灵活

### 4.2 几何体存储

**C++**：使用 union 节省内存
```cpp
struct Geometry {
    Kind kind;
    union {
        struct { ... } pyramid;
        struct { ... } box;
        struct { ... } cylinder;
        // ...
    };
};
```

**Rust**：使用 enum
```rust
pub enum GeometryKind {
    Pyramid(Pyramid),
    Box(Box),
    Cylinder(Cylinder),
    // ...
}
```

**影响**：
- Rust 的 enum 更安全，有类型检查
- C++ 的 union 更紧凑，但需要手动管理类型

---

## 5. 实现优先级建议

### 🔴 **立即实现**（影响正确性）

1. **Snout 的 shear 参数支持**
   - 影响：几何体形状错误
   - 工作量：中等（2-3天）
   - 依赖：无

### 🟠 **近期实现**（影响性能）

2. **连接检测与端盖优化**
   - 影响：三角形数量增加 20-40%
   - 工作量：大（1-2周）
   - 依赖：需要连接数据结构

3. **sphereBasedShape 统一实现**
   - 影响：代码质量和一致性
   - 工作量：小（1-2天）
   - 依赖：无

### 🟡 **中期实现**（改进质量）

4. **自适应环数细分**
   - 影响：球面几何体的三角形质量
   - 工作量：中等（2-3天）
   - 依赖：无

5. **几何体缓存**
   - 影响：重复几何体的性能
   - 工作量：中等（3-5天）
   - 依赖：哈希和比较逻辑

### 🟢 **长期实现**（锦上添花）

6. **Arena 分配器**
   - 影响：内存分配性能
   - 工作量：中等（使用现有 crate）
   - 依赖：无

7. **复杂多边形细分**
   - 影响：FacetGroup 的正确性
   - 工作量：小（集成现有库）
   - 依赖：选择合适的三角化库

8. **小几何体剔除**
   - 影响：大场景的性能
   - 工作量：小（1-2天）
   - 依赖：边界盒计算

---

## 6. 测试建议

### 6.1 单元测试

为每个几何体类型创建测试：
```rust
#[test]
fn test_snout_with_shear() {
    let snout = Snout {
        radius_bottom: 1.0,
        radius_top: 0.5,
        height: 2.0,
        offset_x: 0.1,
        offset_y: 0.1,
        bottom_shear_x: 0.1,  // 约 5.7 度
        bottom_shear_y: 0.0,
        top_shear_x: -0.1,
        top_shear_y: 0.0,
    };
    
    let tri = snout.tessellate(0.01, 1.0);
    
    // 验证顶点数
    assert!(tri.vertices.len() > 0);
    
    // 验证边界盒
    let bbox = compute_bbox(&tri.vertices);
    assert!(bbox.is_valid());
    
    // 验证法线
    for i in 0..tri.normals.len()/3 {
        let n = Vec3::new(
            tri.normals[3*i],
            tri.normals[3*i+1],
            tri.normals[3*i+2],
        );
        assert!((n.length() - 1.0).abs() < 0.01);
    }
}
```

### 6.2 对比测试

与 C++ 实现对比：
```rust
#[test]
fn compare_with_cpp() {
    // 1. 用相同参数生成几何体
    // 2. 导出为 OBJ
    // 3. 比较顶点数、三角形数
    // 4. 比较边界盒
    // 5. 可视化差异
}
```

### 6.3 性能测试

```rust
#[bench]
fn bench_tessellation(b: &mut Bencher) {
    let geo = create_test_geometry();
    b.iter(|| {
        geo.tessellate(0.01, 1.0)
    });
}
```

---

## 7. 文档建议

需要补充的文档：
1. **API 文档**：每个函数的参数说明
2. **示例代码**：常见用法
3. **性能指南**：tolerance 参数的选择
4. **迁移指南**：从 C++ 到 Rust 的差异

---

## 8. 总结

### 当前状态

- ✅ 基础几何体细分：80% 完成
- 🟡 高级功能：40% 完成
- 🔴 性能优化：20% 完成

### 关键差距

1. **Snout shear 参数**：影响正确性，必须实现
2. **连接检测**：影响性能，强烈建议实现
3. **统一球面实现**：影响代码质量，建议实现

### 预估工作量

- 核心功能补齐：2-3 周
- 性能优化：1-2 周
- 测试和文档：1 周

**总计**：4-6 周可以达到与 C++ 实现相当的功能和性能。

---

**文档版本**: 1.0  
**最后更新**: 2024  
**作者**: RVM Parser 项目组
