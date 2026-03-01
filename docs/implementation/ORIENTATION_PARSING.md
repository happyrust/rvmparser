# RVM 解析器 - 方位变换解析文档

## 概述

本文档详细说明 RVM 解析器（C++ 实现）中几何体方位（orientation/transformation）的解析和处理机制。方位信息通过 3x4 仿射变换矩阵表示，包含旋转、缩放、剪切和平移等变换。

## 核心数据结构

### 1. 变换矩阵 `Mat3x4f`

定义位置：`src/LinAlg.h`

```cpp
struct Mat3x4f {
    union {
        struct {
            float m00, m10, m20;  // 第一列：旋转/缩放的 X 轴方向
            float m01, m11, m21;  // 第二列：旋转/缩放的 Y 轴方向
            float m02, m12, m22;  // 第三列：旋转/缩放的 Z 轴方向
            float m03, m13, m23;  // 第四列：平移向量 (tx, ty, tz)
        };
        Vec3f cols[4];            // 按列访问
        float data[12];           // 线性数组访问
    };
};
```

**矩阵布局（列优先）：**
```
[ m00  m01  m02  m03 ]   [ Xx  Yx  Zx  Tx ]
[ m10  m11  m12  m13 ] = [ Xy  Yy  Zy  Ty ]
[ m20  m21  m22  m23 ]   [ Xz  Yz  Zz  Tz ]
```

- 前 3 列：3x3 旋转/缩放矩阵
- 第 4 列：平移向量

### 2. 几何体结构

定义位置：`src/Store.h`

```cpp
struct Geometry {
    Mat3x4f M_3x4;        // 局部到世界的变换矩阵
    BBox3f bboxLocal;     // 局部空间包围盒
    BBox3f bboxWorld;     // 世界空间包围盒（变换后）
    // ... 其他属性
};
```

## RVM 文件方位解析

### 解析流程

实现位置：`src/ParserRVM.cpp` - `parse_prim()` 函数

#### 1. 读取变换矩阵

```cpp
// 从 RVM 二进制文件读取 12 个浮点数
for (unsigned i = 0; i < 12; i++) {
    curr_ptr = read_float32_be(g->M_3x4.data[i], curr_ptr, end_ptr);
}
```

**关键点：**
- 数据格式：32 位浮点数（IEEE 754）
- 字节序：**大端序（Big-Endian）**
- 存储顺序：列优先（column-major）
- 数据量：12 个浮点数 = 48 字节

#### 2. 读取局部包围盒

```cpp
// 读取 6 个浮点数：min(x,y,z) 和 max(x,y,z)
for (unsigned i = 0; i < 6; i++) {
    curr_ptr = read_float32_be(g->bboxLocal.data[i], curr_ptr, end_ptr);
}
```

#### 3. 计算世界空间包围盒

```cpp
// 应用变换矩阵到局部包围盒
g->bboxWorld = transform(g->M_3x4, g->bboxLocal);
```

### 大端序读取实现

```cpp
const char* read_float32_be(float& rv, const char* curr_ptr, const char* end_ptr) {
    union {
        float f;
        uint32_t u;
    };
    
    auto* q = reinterpret_cast<const uint8_t*>(curr_ptr);
    // 大端序：高位字节在前
    u = q[0] << 24 | q[1] << 16 | q[2] << 8 | q[3];
    rv = f;
    return curr_ptr + 4;
}
```

## 坐标变换实现

### 1. 矩阵-向量乘法

实现位置：`src/LinAlgOps.h`

```cpp
inline Vec3f mul(const Mat3x4f& A, const Vec3f& x) {
    Vec3f r;
    for (size_t k = 0; k < 3; k++) {
        r.data[k] = A.data[k]     * x.data[0] +  // 旋转/缩放 X 分量
                    A.data[3 + k] * x.data[1] +  // 旋转/缩放 Y 分量
                    A.data[6 + k] * x.data[2] +  // 旋转/缩放 Z 分量
                    A.data[9 + k];               // 平移分量
    }
    return r;
}
```

**数学表达式：**
```
result = M * point + translation
```

### 2. 包围盒变换

实现位置：`src/LinAlgOps.cpp`

```cpp
BBox3f transform(const Mat3x4f& M, const BBox3f& bbox) {
    // 变换包围盒的 8 个顶点
    const Vec3f p[8] = {
        mul(M, makeVec3f(bbox.min.x, bbox.min.y, bbox.min.z)),
        mul(M, makeVec3f(bbox.min.x, bbox.min.y, bbox.max.z)),
        mul(M, makeVec3f(bbox.min.x, bbox.max.y, bbox.min.z)),
        mul(M, makeVec3f(bbox.min.x, bbox.max.y, bbox.max.z)),
        mul(M, makeVec3f(bbox.max.x, bbox.min.y, bbox.min.z)),
        mul(M, makeVec3f(bbox.max.x, bbox.min.y, bbox.max.z)),
        mul(M, makeVec3f(bbox.max.x, bbox.max.y, bbox.min.z)),
        mul(M, makeVec3f(bbox.max.x, bbox.max.y, bbox.max.z))
    };
    
    // 计算变换后的轴对齐包围盒
    return makeBBox3f(
        min(min(min(p[0], p[1]), min(p[2], p[3])), 
            min(min(p[4], p[5]), min(p[6], p[7]))),
        max(max(max(p[0], p[1]), max(p[2], p[3])), 
            max(max(p[4], p[5]), max(p[6], p[7])))
    );
}
```

**算法说明：**
1. 将局部包围盒的 8 个顶点全部变换到世界空间
2. 计算变换后顶点的最小/最大边界
3. 返回新的轴对齐包围盒（AABB）

### 3. 矩阵运算工具

实现位置：`src/LinAlgOps.cpp`

```cpp
// 矩阵求逆（用于 3x3 旋转矩阵）
Mat3f inverse(const Mat3f& M);

// 矩阵乘法
Mat3f mul(const Mat3f& A, const Mat3f& B);

// 获取缩放因子（取三个轴的最大缩放）
float getScale(const Mat3f& M) {
    const float sx = length(M.cols[0]);
    const float sy = length(M.cols[1]);
    const float sz = length(M.cols[2]);
    return max(sx, max(sy, sz));
}
```

## 层级变换

### 组节点的参考点

实现位置：`src/ParserRVM.cpp` - `parse_cntb()` 函数

```cpp
// 读取组节点的平移参考点
for (unsigned i = 0; i < 3; i++) {
    curr_ptr = read_float32_be(g->group.translation[i], curr_ptr, end_ptr);
    g->group.translation[i] *= 0.001f;  // 单位转换：毫米 → 米
}
```

**重要说明：**
- 这个平移向量是**参考点**，用于定义组的局部坐标系
- 几何体的变换矩阵**不相对于**这个参考点
- 主要用于可视化和编辑器中的对象定位

### 层级结构

```
File (根节点)
└── Model
    └── Group (CNTB)
        ├── translation[3]  // 参考点
        ├── Group (子组)
        │   └── Geometry (PRIM)
        │       └── M_3x4   // 变换矩阵
        └── Geometry (PRIM)
            └── M_3x4
```

## 坐标系转换

### GLTF 导出的 Z-to-Y 转换

实现位置：`src/ExportGLTF.cpp`

RVM 使用 Z-up 坐标系，GLTF 使用 Y-up 坐标系。导出时可选添加旋转：

```cpp
// 绕 X 轴旋转 -90° 将 +Z 映射到 +Y
// 使用四元数表示：q = (sin(θ/2) * axis, cos(θ/2))
// θ = -90°, axis = [1, 0, 0]

rj::Value rotation(rj::kArrayType);
rotation.PushBack(std::sin(-M_PI_4), alloc);  // x = sin(-π/4) ≈ -0.707
rotation.PushBack(0.f, alloc);                // y = 0
rotation.PushBack(0.f, alloc);                // z = 0
rotation.PushBack(std::cos(-M_PI_4), alloc);  // w = cos(-π/4) ≈ 0.707
```

**旋转矩阵等价形式：**
```
[ 1   0   0 ]
[ 0   0   1 ]
[ 0  -1   0 ]
```

### 导出时的矩阵处理

```cpp
// 将 3x4 矩阵转换为 GLTF 的 4x4 矩阵（列优先）
rj::Value matrix(rj::kArrayType);
for (size_t c = 0; c < 3; c++) {
    for (size_t r = 0; r < 3; r++) {
        matrix.PushBack(geo->M_3x4.cols[c][r], alloc);
    }
    matrix.PushBack(0.f, alloc);  // 第四行前三列为 0
}
for (size_t r = 0; r < 3; r++) {
    // 平移向量减去模型原点
    matrix.PushBack(geo->M_3x4.cols[3][r] - model.origin[r], alloc);
}
matrix.PushBack(1.f, alloc);  // 第四行第四列为 1
```

## ATT 文件处理

实现位置：`src/ParserAtt.cpp`

ATT 文件是**文本格式**的属性文件，**不包含方位矩阵**。

### 文件格式

```
NEW Group1
    attribute1 := 'value1'
    attribute2 := 'value2' &end&
    NEW SubGroup
        attribute3 := 'value3'
    END
END
```

### 解析特点

- 使用 `NEW` 和 `END` 标签定义层级结构
- 使用 `:=` 语法定义键值对属性
- 主要用于附加元数据（材质、标签等）
- 不涉及几何变换信息

## 数据流程图

```
RVM 二进制文件
    ↓
[读取 12 个 float32 (Big-Endian)]
    ↓
Mat3x4f 变换矩阵
    ↓
[应用到局部包围盒]
    ↓
世界空间包围盒
    ↓
[导出时可选坐标系转换]
    ↓
GLTF/OBJ 等格式
```

## 性能考虑

### 1. 内存布局

- 使用 `union` 提供多种访问方式（列、数组、命名字段）
- 列优先存储便于 SIMD 优化
- 紧凑的数据结构减少缓存未命中

### 2. 计算优化

- 包围盒变换只计算 8 个顶点（而非所有几何顶点）
- 延迟计算：只在需要时进行坐标变换
- 矩阵运算使用内联函数减少函数调用开销

### 3. 精度处理

- 使用 `float` (32位) 平衡精度和性能
- 导出时可选使用 `double` (64位) 提高精度
- 单位转换：毫米 → 米（`* 0.001f`）

## 常见问题

### Q1: 为什么使用 3x4 而不是 4x4 矩阵？

**A:** 仿射变换的最后一行总是 `[0, 0, 0, 1]`，存储 3x4 可节省 4 个浮点数（16 字节）。

### Q2: 如何处理非均匀缩放？

**A:** 变换矩阵的三列向量长度不同时表示非均匀缩放。使用 `getScale()` 获取最大缩放因子。

### Q3: 组节点的 translation 如何使用？

**A:** 它是参考点，不影响子对象的变换。主要用于编辑器中的对象定位和可视化。

### Q4: 如何验证矩阵是否正确？

**A:** 检查：
- 行列式不为零（可逆）
- 三个列向量近似正交（纯旋转时）
- 列向量长度表示缩放因子

## 相关文件

- `src/LinAlg.h` - 数学数据结构定义
- `src/LinAlgOps.h` / `.cpp` - 矩阵运算实现
- `src/ParserRVM.cpp` - RVM 文件解析
- `src/Store.h` - 几何体数据存储
- `src/ExportGLTF.cpp` - GLTF 导出（含坐标系转换）

## 参考资料

- RVM 文件格式规范
- 仿射变换数学基础
- GLTF 2.0 规范（坐标系定义）
