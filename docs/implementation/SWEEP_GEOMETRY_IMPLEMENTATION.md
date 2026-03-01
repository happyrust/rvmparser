# RVM Sweep Geometry 实现分析

## 概述

本文档详细分析 RVM 格式中两种扫掠几何体的实现：
- **RPATH (RectangularTorus)**: 矩形截面沿圆弧扫掠
- **GENSEC (CircularTorus)**: 圆形截面沿圆弧扫掠

这两种几何体都采用 **Sweep（扫掠）** 技术生成三角网格，即沿着一条路径移动一个截面形状来创建 3D 体。

---

## 1. RectangularTorus (RPATH) - 矩形截面扫掠

### 1.1 几何定义

```
参数：
- inner_radius: 内半径
- outer_radius: 外半径  
- height: 截面高度
- angle: 扫掠角度（弧度）
```

### 1.2 截面定义

矩形截面由 4 个角点定义（局部坐标系）：

```cpp
float h2 = 0.5f * tor.height;
float square[4][2] = {
    { tor.outer_radius, -h2 },  // 点0: 外半径，底部
    { tor.inner_radius, -h2 },  // 点1: 内半径，底部
    { tor.inner_radius,  h2 },  // 点2: 内半径，顶部
    { tor.outer_radius,  h2 },  // 点3: 外半径，顶部
};
```

截面形状（俯视图）：
```
     3 -------- 2
     |          |
     |          |  ← height
     |          |
     0 -------- 1
     
     ↑          ↑
  outer_r    inner_r
```

### 1.3 扫掠路径采样

```cpp
// 基于弦高误差的自适应采样
unsigned segments = sagittaBasedSegmentCount(tor.angle, tor.outer_radius, scale);
unsigned samples = segments + 1;  // 开放路径需要额外采样点

// 预计算每个采样点的角度
for (unsigned i = 0; i < samples; i++) {
    t0[2*i + 0] = std::cos((tor.angle / segments) * i);
    t0[2*i + 1] = std::sin((tor.angle / segments) * i);
}
```

**弦高（Sagitta）采样原理**：
```
弦高 = radius * (1 - cos(arc/segments))

采样数 = arc / acos(1 - tolerance/radius)
```

这确保了曲线的离散化误差不超过指定的容差。

### 1.4 顶点生成策略

C++ 实现采用 **面导向** 的顶点生成策略：

```cpp
// 矩形有 4 个面：底面、内侧面、顶面、外侧面
for (unsigned i = 0; i < samples; i++) {
    float n[4][3] = {
        { 0.f, 0.f, -1.f },                    // 面0: 底面法线 (向下)
        { -t0[2*i+0], -t0[2*i+1], 0.f },      // 面1: 内侧面法线 (向内)
        { 0.f, 0.f, 1.f },                     // 面2: 顶面法线 (向上)
        { t0[2*i+0], t0[2*i+1], 0.f },        // 面3: 外侧面法线 (向外)
    };

    // 每个面生成 2 个顶点（边的两个端点）
    for (unsigned k = 0; k < 4; k++) {
        unsigned kk = (k + 1) & 3;  // 下一个角点索引
        
        // 顶点1: 当前角点
        vertex(normals, vertices, l, 
               n[k], 
               square[k][0] * t0[2*i+0],    // x = radius * cos(θ)
               square[k][0] * t0[2*i+1],    // z = radius * sin(θ)
               square[k][1]);                // y = height_offset
        
        // 顶点2: 下一个角点
        vertex(normals, vertices, l,
               n[k],
               square[kk][0] * t0[2*i+0],
               square[kk][0] * t0[2*i+1],
               square[kk][1]);
    }
}
```

**顶点布局**（每个采样点 8 个顶点）：
```
采样点 i 的顶点索引：
  base = i * 8
  
  面0 (底面):   base+0, base+1  (点0→点1)
  面1 (内侧):   base+2, base+3  (点1→点2)
  面2 (顶面):   base+4, base+5  (点2→点3)
  面3 (外侧):   base+6, base+7  (点3→点0)
```

### 1.5 索引生成

```cpp
// 为每个面生成四边形（2个三角形）
for (unsigned i = 0; i + 1 < samples; i++) {
    for (unsigned k = 0; k < 4; k++) {
        // 当前采样点的面 k
        unsigned v0 = 4*2*(i+0) + 0 + 2*k;
        unsigned v1 = 4*2*(i+0) + 1 + 2*k;
        // 下一个采样点的面 k
        unsigned v2 = 4*2*(i+1) + 0 + 2*k;
        unsigned v3 = 4*2*(i+1) + 1 + 2*k;
        
        // 三角形1
        indices[l++] = v0;
        indices[l++] = v1;
        indices[l++] = v2;
        
        // 三角形2
        indices[l++] = v2;
        indices[l++] = v1;
        indices[l++] = v3;
    }
}
```

### 1.6 端盖生成

```cpp
bool cap[2] = { true, true };

// 检测连接，避免重复端盖
for (unsigned i = 0; i < 2; i++) {
    auto * con = geo->connections[i];
    if (con && con->flags == Connection::Flags::HasRectangularSide) {
        if (doInterfacesMatch(geo, con)) {
            cap[i] = false;
            discardedCaps++;
        }
    }
}

// 起始端盖 (i=0)
if (cap[0]) {
    for (unsigned k = 0; k < 4; k++) {
        vertex(normals, vertices, l,
               0.f, -1.f, 0.f,  // 法线指向负Y方向
               square[k][0] * t0[0],
               square[k][0] * t0[1],
               square[k][1]);
    }
    // 生成2个三角形
    quadIndices(indices, l, offset, 0, 2, 1, 3);
}

// 结束端盖 (i=samples-1)
if (cap[1]) {
    unsigned m = 2 * (samples - 1);
    for (unsigned k = 0; k < 4; k++) {
        vertex(normals, vertices, l,
               -t0[m+1], t0[m+0], 0.f,  // 法线垂直于扫掠方向
               square[k][0] * t0[m+0],
               square[k][0] * t0[m+1],
               square[k][1]);
    }
    quadIndices(indices, l, offset, 0, 1, 2, 3);
}
```

---

## 2. CircularTorus (GENSEC) - 圆形截面扫掠

### 2.1 几何定义

```
参数：
- offset: 主半径（从中心到截面圆心的距离）
- radius: 副半径（截面圆的半径）
- angle: 扫掠角度（弧度）
```

### 2.2 双参数曲面

CircularTorus 是一个 **双参数曲面**，需要两层循环：

```cpp
// 主方向（环向/toroidal）- 沿扫掠路径
unsigned segments_l = sagittaBasedSegmentCount(
    ct.angle, 
    ct.offset + ct.radius,  // 使用外半径
    scale
);
unsigned samples_l = segments_l + 1;  // 开放路径

// 副方向（极向/poloidal）- 截面圆
unsigned segments_s = sagittaBasedSegmentCount(
    2π, 
    ct.radius, 
    scale
);
unsigned samples_s = segments_s;  // 闭合圆
```

### 2.3 参数方程

```cpp
// 主方向角度 u ∈ [0, angle]
// 副方向角度 v ∈ [0, 2π]

for (unsigned u = 0; u < samples_l; u++) {
    float theta = (ct.angle / (samples_l - 1)) * u;
    
    for (unsigned v = 0; v < samples_s; v++) {
        float phi = (2π / samples_s) * v;
        
        // 位置向量
        Vec3 p(
            (ct.offset + ct.radius * cos(phi)) * cos(theta),
            ct.radius * sin(phi),
            (ct.offset + ct.radius * cos(phi)) * sin(theta)
        );
        
        // 法线向量（指向截面圆的径向）
        Vec3 n(
            cos(phi) * cos(theta),
            sin(phi),
            cos(phi) * sin(theta)
        );
        
        addVertex(p, n);
    }
}
```

**几何解释**：
1. 先在 XZ 平面上放置一个半径为 `offset` 的圆（主圆）
2. 在主圆上的每个点，放置一个半径为 `radius` 的小圆（截面圆）
3. 小圆的圆心沿着主圆移动，形成圆环面

### 2.4 顶点布局

```
顶点索引 = u * samples_s + v

其中：
  u: 主方向索引 [0, samples_l)
  v: 副方向索引 [0, samples_s)
```

### 2.5 索引生成

```cpp
for (unsigned u = 0; u + 1 < samples_l; u++) {
    for (unsigned v = 0; v + 1 < samples_s; v++) {
        unsigned v00 = samples_s * (u + 0) + (v + 0);
        unsigned v01 = samples_s * (u + 0) + (v + 1);
        unsigned v10 = samples_s * (u + 1) + (v + 0);
        unsigned v11 = samples_s * (u + 1) + (v + 1);
        
        // 四边形的两个三角形
        indices[l++] = v00;
        indices[l++] = v10;
        indices[l++] = v11;
        
        indices[l++] = v11;
        indices[l++] = v01;
        indices[l++] = v00;
    }
    
    // 闭合副方向（v = samples_s-1 连接到 v = 0）
    unsigned v00 = samples_s * (u + 0) + (samples_s - 1);
    unsigned v01 = samples_s * (u + 0) + 0;
    unsigned v10 = samples_s * (u + 1) + (samples_s - 1);
    unsigned v11 = samples_s * (u + 1) + 0;
    
    indices[l++] = v00;
    indices[l++] = v10;
    indices[l++] = v11;
    
    indices[l++] = v11;
    indices[l++] = v01;
    indices[l++] = v00;
}
```

### 2.6 圆形端盖

端盖是圆形的，需要特殊的三角化策略：

```cpp
// 扇形三角化（Fan Triangulation）
unsigned tessellateCircle(uint32_t* indices, unsigned l, 
                         uint32_t* t, uint32_t* src, unsigned N) {
    while (3 <= N) {
        unsigned m = 0;
        unsigned i;
        // 每次迭代消耗相邻的3个顶点
        for (i = 0; i + 2 < N; i += 2) {
            indices[l++] = src[i];
            indices[l++] = src[i + 1];
            indices[l++] = src[i + 2];
            t[m++] = src[i];
        }
        // 剩余顶点
        for (; i < N; i++) {
            t[m++] = src[i];
        }
        N = m;
        std::swap(t, src);
    }
    return l;
}

// 起始端盖
if (cap[0]) {
    for (unsigned v = 0; v < samples_s; v++) {
        vertex(normals, vertices, l,
               0.f, -1.f, 0.f,
               (ct.radius * t1[2*v+0] + ct.offset) * t0[0],
               (ct.radius * t1[2*v+0] + ct.offset) * t0[1],
               ct.radius * t1[2*v+1]);
    }
    // 构建顶点环
    for (unsigned i = 0; i < samples_s; i++) {
        u1[i] = offset + (samples_s - 1) - i;  // 反向
    }
    tessellateCircle(indices, l, u2.data(), u1.data(), samples_s);
}

// 结束端盖
if (cap[1]) {
    unsigned m = 2 * (samples_l - 1);
    for (unsigned v = 0; v < samples_s; v++) {
        vertex(normals, vertices, l,
               -t0[m+1], t0[m+0], 0.f,
               (ct.radius * t1[2*v+0] + ct.offset) * t0[m+0],
               (ct.radius * t1[2*v+0] + ct.offset) * t0[m+1],
               ct.radius * t1[2*v+1]);
    }
    // 构建顶点环
    for (unsigned i = 0; i < samples_s; i++) {
        u1[i] = offset + i;  // 正向
    }
    tessellateCircle(indices, l, u2.data(), u1.data(), samples_s);
}
```

---

## 3. 关键技术细节

### 3.1 弦高（Sagitta）自适应采样

```cpp
unsigned sagittaBasedSegmentCount(float arc, float radius, float scale) {
    // 弦高公式：s = r * (1 - cos(θ))
    // 要求：s ≤ tolerance
    // 推导：θ ≤ acos(1 - tolerance/r)
    
    float samples = arc / std::acos(
        std::max(-1.f, 1.f - tolerance / (scale * radius))
    );
    
    return std::min(maxSamples, 
           unsigned(std::max(float(minSamples), std::ceil(samples))));
}
```

**优点**：
- 自动适应曲率：曲率大的地方采样密，曲率小的地方采样疏
- 保证几何误差：离散化误差不超过 `tolerance`
- 性能优化：避免过度细分

### 3.2 连接检测与端盖优化

```cpp
bool doInterfacesMatch(const Geometry* geo, const Connection* con) {
    bool isFirst = geo == con->geo[0];
    
    auto thisIFace = getInterface(geo, con->offset[isFirst ? 0 : 1]);
    auto thatIFace = getInterface(con->geo[isFirst ? 1 : 0], 
                                   con->offset[isFirst ? 1 : 0]);
    
    if (thisIFace.kind != thatIFace.kind) return false;
    
    if (thisIFace.kind == Interface::Kind::Circular) {
        // 圆形接口：比较半径
        return thisIFace.circular.radius <= 1.05f * thatIFace.circular.radius;
    }
    else {
        // 矩形接口：比较4个角点
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
```

**优化效果**：
- 避免重复几何：相邻几何体的接触面不生成端盖
- 减少三角形数量：典型场景可减少 20-40% 的三角形
- 提高渲染性能：减少 overdraw

### 3.3 法线计算

**RectangularTorus**：
- 底面/顶面：垂直于 Y 轴 `(0, ±1, 0)`
- 内侧面：指向圆心 `(-cos(θ), -sin(θ), 0)`
- 外侧面：背离圆心 `(cos(θ), sin(θ), 0)`

**CircularTorus**：
- 壳体：指向截面圆的径向 `(cos(φ)cos(θ), sin(φ), cos(φ)sin(θ))`
- 起始端盖：垂直于扫掠起点 `(0, -1, 0)`
- 结束端盖：垂直于扫掠终点 `(-sin(θ_end), 0, cos(θ_end))`

---

## 4. 性能考虑

### 4.1 内存布局

C++ 实现使用 **交错数组**（Interleaved Arrays）：

```cpp
struct Triangulation {
    float* vertices;   // [x0, y0, z0, x1, y1, z1, ...]
    float* normals;    // [nx0, ny0, nz0, nx1, ny1, nz1, ...]
    uint32_t* indices; // [i0, i1, i2, i3, i4, i5, ...]
};
```

**优点**：
- 缓存友好：顶点数据连续存储
- GPU 友好：适合现代图形 API（OpenGL/Vulkan）
- 减少内存碎片

### 4.2 采样数量控制

```cpp
minSamples = 3;   // 最少采样数，保证基本形状
maxSamples = 64;  // 最多采样数，避免过度细分
```

典型采样数：
- 小半径（< 1m）：8-16 个采样点
- 中等半径（1-10m）：16-32 个采样点
- 大半径（> 10m）：32-64 个采样点

### 4.3 三角形数量估算

**RectangularTorus**：
```
壳体三角形 = 4 * 2 * segments
端盖三角形 = 2 * 2 (每个端盖2个三角形)
总计 ≈ 8 * segments + 4
```

**CircularTorus**：
```
壳体三角形 = 2 * segments_l * segments_s
端盖三角形 = 2 * (segments_s - 2) (扇形三角化)
总计 ≈ 2 * segments_l * segments_s + 2 * segments_s
```

---

## 5. 坐标系统

### 5.1 局部坐标系

```
Y 轴：高度方向（向上）
X 轴：初始扫掠方向
Z 轴：右手坐标系
```

### 5.2 扫掠方向

```
起始角度：0°（沿 +X 轴）
扫掠方向：逆时针（从 +Y 轴向下看）
角度范围：[0, angle]
```

### 5.3 世界坐标变换

```cpp
// 局部坐标 → 世界坐标
Vec3 worldPos = mul(geo->M_3x4, localPos);
Vec3 worldNormal = mul(geo->M_3x4.rotation(), localNormal);
```

---

## 6. 与 Rust 实现的对比

| 特性 | C++ 实现 | Rust 实现 |
|------|---------|----------|
| 顶点生成 | 面导向（每面2顶点） | 面导向（匹配C++） |
| 索引生成 | 显式循环 | 显式循环 |
| 端盖检测 | 连接检测 + 接口匹配 | 简化版（总是生成） |
| 内存管理 | Arena 分配器 | Vec 动态数组 |
| 采样策略 | Sagitta 自适应 | Sagitta 自适应 |
| 法线计算 | 手动计算 | 手动计算 |

**主要差异**：
1. Rust 版本暂未实现连接检测优化
2. Rust 使用更安全的内存管理（Vec vs 裸指针）
3. C++ 版本有更多的性能优化（预分配、缓存）

---

## 7. 调试技巧

### 7.1 可视化验证

```cpp
// 添加调试线
store->addDebugLine(p0.data, p1.data, 0xff0000);  // 红色
store->addDebugLine(p0.data, p1.data, 0x00ff00);  // 绿色
store->addDebugLine(p0.data, p1.data, 0x0000ff);  // 蓝色
```

### 7.2 断言检查

```cpp
assert(l == 3 * tri->vertices_n);   // 顶点数正确
assert(l == 3 * tri->triangles_n);  // 索引数正确
assert(o == tri->vertices_n);       // 偏移量正确
```

### 7.3 边界盒验证

```cpp
BBox3f box = createEmptyBBox3f();
for (unsigned i = 0; i < tri->vertices_n; i++) {
    engulf(box, makeVec3f(tri->vertices + 3 * i));
}
// 验证生成的边界盒与预期的边界盒匹配
```

---

## 8. 参考资料

- RVM 文件格式规范
- 计算机图形学：扫掠曲面生成
- 自适应曲线细分算法
- 三角网格拓扑优化

---

**文档版本**: 1.0  
**最后更新**: 2024  
**作者**: RVM Parser 项目组
