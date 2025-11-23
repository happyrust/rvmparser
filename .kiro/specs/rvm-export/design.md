# Design Document

## Overview

本设计文档描述了为 Rust RVM 解析器添加导出功能的架构和实现方案。导出功能将支持三种主要格式：OBJ（带 MTL）、JSON 和 GLTF/GLB。设计遵循以下原则：

1. **访问者模式**：使用现有的 Visitor trait 实现导出器
2. **流式写入**：避免在内存中构建完整的输出
3. **模块化设计**：每种导出格式独立实现
4. **曲面细分**：实现基本几何体到三角网格的转换
5. **配置灵活性**：支持多种导出选项

## Architecture

### 模块结构

```
rvm-rs/
├── src/
│   ├── export/
│   │   ├── mod.rs           # 导出模块入口
│   │   ├── obj.rs           # OBJ 导出器
│   │   ├── json.rs          # JSON 导出器
│   │   ├── gltf.rs          # GLTF/GLB 导出器
│   │   └── tessellator.rs   # 曲面细分器
│   ├── visitor/
│   │   ├── mod.rs           # 访问者模块（已存在）
│   │   └── stats.rs         # 统计访问者（已存在）
│   └── main.rs              # 命令行入口（需更新）
```

### 依赖关系

- `export/obj` → `visitor`, `store`, `export/tessellator`
- `export/json` → `visitor`, `store`
- `export/gltf` → `visitor`, `store`, `export/tessellator`
- `export/tessellator` → `store/geometry`, `math`

## Components and Interfaces

### 1. Tessellator 模块 (`export/tessellator.rs`)

将基本几何体转换为三角网格：

```rust
pub struct Triangulation {
    pub vertices: Vec<f32>,      // 顶点坐标 (x, y, z)
    pub normals: Vec<f32>,       // 法线向量 (nx, ny, nz)
    pub indices: Vec<u32>,       // 三角形索引
    pub error: f32,              // 细分误差估计
}

pub trait Tessellate {
    fn tessellate(&self, tolerance: f32) -> Triangulation;
}

// 为每种几何类型实现 Tessellate
impl Tessellate for Cylinder { ... }
impl Tessellate for Sphere { ... }
impl Tessellate for Box { ... }
// ... 其他几何类型
```

### 2. OBJ 导出器 (`export/obj.rs`)

```rust
pub struct ObjExporter {
    obj_file: BufWriter<File>,
    mtl_file: BufWriter<File>,
    defined_materials: HashSet<u32>,
    vertex_offset: u32,
    normal_offset: u32,
    current_path: Vec<String>,
    options: ObjExportOptions,
}

pub struct ObjExportOptions {
    pub include_normals: bool,
    pub group_bounding_boxes: bool,
}

impl ObjExporter {
    pub fn new(obj_path: &str, mtl_path: &str, options: ObjExportOptions) 
        -> Result<Self, ExportError>;
    
    fn write_material(&mut self, color: u32, transparency: u8) 
        -> Result<(), ExportError>;
    
    fn write_triangulation(&mut self, tri: &Triangulation, transform: &Affine3A) 
        -> Result<(), ExportError>;
}

impl Visitor for ObjExporter {
    fn visit_node(&mut self, node: &Node, store: &Store);
    fn visit_geometry(&mut self, geometry: &Geometry, store: &Store);
}
```

### 3. JSON 导出器 (`export/json.rs`)

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct JsonNode {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bbox: Option<[f32; 6]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<JsonNode>,
}

pub struct JsonExporter {
    root_nodes: Vec<JsonNode>,
    node_stack: Vec<JsonNode>,
}

impl JsonExporter {
    pub fn new() -> Self;
    
    pub fn write_to_file(&self, path: &str) -> Result<(), ExportError>;
}

impl Visitor for JsonExporter {
    fn visit_node(&mut self, node: &Node, store: &Store);
    fn visit_geometry(&mut self, geometry: &Geometry, store: &Store);
}
```

### 4. GLTF 导出器 (`export/gltf.rs`)

```rust
use serde_json::Value as JsonValue;

pub struct GltfExporter {
    nodes: Vec<JsonValue>,
    meshes: Vec<JsonValue>,
    accessors: Vec<JsonValue>,
    buffer_views: Vec<JsonValue>,
    materials: Vec<JsonValue>,
    buffers: Vec<Vec<u8>>,
    defined_materials: HashMap<u64, u32>,
    options: GltfExportOptions,
}

pub struct GltfExportOptions {
    pub center_model: bool,
    pub rotate_z_to_y: bool,
    pub include_attributes: bool,
    pub merge_geometries: bool,
    pub binary_format: bool,  // GLB vs GLTF
}

impl GltfExporter {
    pub fn new(options: GltfExportOptions) -> Self;
    
    fn create_accessor_vec3(&mut self, data: &[f32]) -> u32;
    fn create_accessor_indices(&mut self, data: &[u32]) -> u32;
    fn create_material(&mut self, color: u32, transparency: u8) -> u32;
    
    pub fn write_to_file(&self, path: &str) -> Result<(), ExportError>;
    fn write_gltf(&self, path: &str) -> Result<(), ExportError>;
    fn write_glb(&self, path: &str) -> Result<(), ExportError>;
}

impl Visitor for GltfExporter {
    fn visit_node(&mut self, node: &Node, store: &Store);
    fn visit_geometry(&mut self, geometry: &Geometry, store: &Store);
}
```

### 5. 导出错误类型 (`export/mod.rs`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Invalid export path: {0}")]
    InvalidPath(String),
    
    #[error("Tessellation error: {0}")]
    Tessellation(String),
}
```

## Data Models

### 曲面细分数据

```rust
pub struct Triangulation {
    pub vertices: Vec<f32>,   // [x, y, z, x, y, z, ...]
    pub normals: Vec<f32>,    // [nx, ny, nz, nx, ny, nz, ...]
    pub indices: Vec<u32>,    // [i0, i1, i2, i0, i1, i2, ...]
    pub error: f32,
}
```

### GLTF 缓冲区数据

```rust
struct BufferData {
    data: Vec<u8>,
    offset: u32,
    length: u32,
}

struct Accessor {
    buffer_view: u32,
    component_type: u32,
    count: u32,
    type_: String,  // "VEC3", "SCALAR", etc.
    min: Vec<f32>,
    max: Vec<f32>,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: OBJ file creation

*For any* valid export path with .obj extension, the system should create both .obj and .mtl files
**Validates: Requirements 1.1**

### Property 2: Triangulation vertex count

*For any* tessellated geometry, the number of vertices should be divisible by 3 (since each vertex has x, y, z)
**Validates: Requirements 4.1**

### Property 3: Material uniqueness

*For any* two geometries with the same color and transparency, they should reference the same material ID
**Validates: Requirements 9.1**

### Property 4: Scene graph hierarchy preservation

*For any* exported JSON, the parent-child relationships should match the original scene graph structure
**Validates: Requirements 2.2**

### Property 5: GLTF accessor bounds

*For any* GLTF accessor, the min and max values should correctly bound all data in the accessor
**Validates: Requirements 10.2**

### Property 6: Export format detection

*For any* file path ending in .gltf, the system should export in GLTF text format, and for .glb, in binary format
**Validates: Requirements 3.1, 3.2**

### Property 7: Vertex offset consistency

*For any* OBJ export, face indices should correctly reference vertices using cumulative offsets
**Validates: Requirements 1.5**

### Property 8: Normal vector normalization

*For any* exported normal vector, its length should be approximately 1.0 (unit vector)
**Validates: Requirements 4.5**

### Property 9: Command-line export option handling

*For any* valid export command-line argument (--export-obj, --export-json, --export-gltf), the system should perform the corresponding export
**Validates: Requirements 7.1, 7.2, 7.3**

### Property 10: Material transparency mapping

*For any* geometry with transparency value T, the exported material should have alpha = 1.0 - T/100.0
**Validates: Requirements 9.3**

## Error Handling

### 导出错误

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Invalid export path: {0}")]
    InvalidPath(String),
    
    #[error("Tessellation error: {0}")]
    Tessellation(String),
    
    #[error("Unsupported geometry type: {0}")]
    UnsupportedGeometry(String),
}
```

### 错误处理策略

- 文件 I/O 错误立即返回
- 几何体细分失败时跳过该几何体并记录警告
- 无效的导出选项在命令行解析时捕获
- 使用 `Result` 类型进行错误传播

## Testing Strategy

### 单元测试

**曲面细分测试**:
- 测试每种几何类型的细分
- 验证顶点数量正确
- 验证法线向量归一化
- 测试边界情况（零尺寸、负值）

**导出器测试**:
- 测试材质去重
- 测试顶点偏移计算
- 测试场景图遍历
- 测试文件写入

### 集成测试

**端到端导出测试**:
- 解析测试 RVM 文件
- 导出为各种格式
- 验证输出文件存在
- 验证输出文件可被外部工具读取

**格式验证**:
- 使用 OBJ 验证器验证 OBJ 输出
- 使用 JSON schema 验证 JSON 输出
- 使用 GLTF 验证器验证 GLTF 输出

### 性能测试

- 测试大型模型的导出时间
- 测试内存使用
- 测试流式写入的效率

## Implementation Notes

### 曲面细分策略

1. **圆柱体**: 使用参数化方法，沿圆周和高度方向细分
2. **球体**: 使用经纬度网格或 icosphere 细分
3. **盒子**: 直接生成 12 个三角形（每面 2 个）
4. **圆环**: 使用双参数化方法
5. **复杂几何体**: 根据曲率自适应细分

### OBJ 导出注意事项

1. **顶点索引**: OBJ 使用 1-based 索引，需要转换
2. **材质库**: MTL 文件路径使用相对路径
3. **对象分组**: 使用 `o` 命令标记不同的组
4. **法线**: 使用 `vn` 命令，面引用格式为 `v/vt/vn`

### GLTF 导出注意事项

1. **数据对齐**: 所有缓冲区数据必须 4 字节对齐
2. **坐标系**: GLTF 使用右手坐标系，Y 轴向上
3. **材质**: 使用 PBR 金属粗糙度工作流
4. **GLB 格式**: 包含 JSON 块和二进制块，需要正确的块头

### 性能优化

1. **批量写入**: 使用 `BufWriter` 减少系统调用
2. **几何合并**: 合并相同材质的几何体减少绘制调用
3. **内存池**: 重用临时缓冲区避免频繁分配
4. **并行处理**: 可选地并行细分独立的几何体

## Dependencies

```toml
[dependencies]
# 现有依赖
glam = "0.30"
nom = "8.0"
memmap2 = "0.9"
thiserror = "2.0"

# 新增依赖
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"
```

## Build and Development

### 开发流程

1. 实现 Tessellator 模块
2. 实现 OBJ 导出器
3. 实现 JSON 导出器
4. 实现 GLTF 导出器
5. 更新命令行接口
6. 添加集成测试
7. 性能优化

### 测试命令

```bash
# 单元测试
cargo test --lib

# 集成测试
cargo test --test export_tests

# 测试 OBJ 导出
cargo run -- input.rvm --export-obj output.obj

# 测试 JSON 导出
cargo run -- input.rvm --export-json output.json

# 测试 GLTF 导出
cargo run -- input.rvm --export-gltf output.gltf

# 测试 GLB 导出
cargo run -- input.rvm --export-gltf output.glb
```
