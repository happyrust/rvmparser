# Design Document

## Overview

本设计文档描述了将 C++ rvmparser 移植到 Rust 的架构和实现方案。移植将保持原有功能的同时，充分利用 Rust 的类型系统、所有权模型和现代库生态系统。核心设计原则包括：

1. **类型安全**：使用 Rust 的强类型系统和枚举来表示不同的节点和几何类型
2. **零拷贝解析**：利用 nom 和内存映射实现高效的二进制解析
3. **内存效率**：使用 Arena 分配器和字符串驻留减少内存开销
4. **可组合性**：使用 nom 组合子构建模块化的解析器
5. **可测试性**：将解析逻辑与 I/O 分离，便于单元测试

## Architecture

### 模块结构

```
rvm-rs/
├── src/
│   ├── main.rs           # 命令行入口
│   ├── lib.rs            # 库入口
│   ├── parser/
│   │   ├── mod.rs        # 解析器模块入口
│   │   ├── rvm.rs        # RVM 二进制解析器
│   │   ├── att.rs        # 属性文件解析器
│   │   └── common.rs     # 通用解析组合子
│   ├── store/
│   │   ├── mod.rs        # 数据存储模块入口
│   │   ├── node.rs       # 场景图节点
│   │   ├── geometry.rs   # 几何体定义
│   │   ├── arena.rs      # Arena 分配器
│   │   └── strings.rs    # 字符串驻留
│   ├── math/
│   │   ├── mod.rs        # 数学工具模块
│   │   └── bbox.rs       # 包围盒计算
│   └── visitor/
│       ├── mod.rs        # 访问者模式接口
│       └── stats.rs      # 统计信息收集
```

### 依赖关系

- `main.rs` → `parser`, `store`
- `parser` → `store`, `math`
- `store` → `math`
- `visitor` → `store`

## Components and Interfaces

### 1. Parser 模块

#### RVM 解析器 (`parser/rvm.rs`)

使用 nom 组合子解析 RVM 二进制格式：

```rust
pub fn parse_rvm<'a>(input: &'a [u8], store: &mut Store) -> IResult<&'a [u8], ()>;

// 块解析器
fn parse_chunk_header(input: &[u8]) -> IResult<&[u8], ChunkHeader>;
fn parse_head(input: &[u8]) -> IResult<&[u8], HeadData>;
fn parse_modl(input: &[u8]) -> IResult<&[u8], ModlData>;
fn parse_cntb(input: &[u8]) -> IResult<&[u8], CntbData>;
fn parse_prim(input: &[u8]) -> IResult<&[u8], PrimData>;
fn parse_colr(input: &[u8]) -> IResult<&[u8], ColrData>;

// 基本类型解析器
fn be_u32(input: &[u8]) -> IResult<&[u8], u32>;
fn be_f32(input: &[u8]) -> IResult<&[u8], f32>;
fn parse_string(input: &[u8]) -> IResult<&[u8], &str>;
fn parse_mat3x4(input: &[u8]) -> IResult<&[u8], glam::Affine3A>;
```

#### 属性解析器 (`parser/att.rs`)

解析文本格式的属性文件：

```rust
pub fn parse_att<'a>(input: &'a str, store: &mut Store) -> Result<(), ParseError>;

// 行解析器
fn parse_new_line(input: &str) -> IResult<&str, &str>;
fn parse_end_line(input: &str) -> IResult<&str, ()>;
fn parse_attribute_line(input: &str) -> IResult<&str, Vec<(&str, &str)>>;
fn parse_indentation(input: &str) -> IResult<&str, usize>;
```

### 2. Store 模块

#### 节点定义 (`store/node.rs`)

```rust
pub enum NodeKind {
    File(FileNode),
    Model(ModelNode),
    Group(GroupNode),
}

pub struct Node {
    pub kind: NodeKind,
    pub next: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
}

pub struct FileNode {
    pub info: StringId,
    pub note: StringId,
    pub date: StringId,
    pub user: StringId,
    pub encoding: StringId,
    pub path: StringId,
}

pub struct ModelNode {
    pub project: StringId,
    pub name: StringId,
    pub first_color: Option<ColorId>,
}

pub struct GroupNode {
    pub name: StringId,
    pub translation: glam::Vec3,
    pub material: u32,
    pub transparency: u32,
    pub bbox_world: BBox3,
    pub first_geometry: Option<GeometryId>,
    pub attributes: Vec<Attribute>,
}

pub struct Attribute {
    pub key: StringId,
    pub value: StringId,
}
```

#### 几何体定义 (`store/geometry.rs`)

```rust
pub enum GeometryKind {
    Pyramid(Pyramid),
    Box(Box),
    RectangularTorus(RectangularTorus),
    CircularTorus(CircularTorus),
    EllipticalDish(EllipticalDish),
    SphericalDish(SphericalDish),
    Snout(Snout),
    Cylinder(Cylinder),
    Sphere(Sphere),
    Line(Line),
    FacetGroup(FacetGroup),
}

pub struct Geometry {
    pub kind: GeometryKind,
    pub geo_type: GeometryType,
    pub transform: glam::Affine3A,
    pub bbox_local: BBox3,
    pub bbox_world: BBox3,
    pub color: u32,
    pub transparency: u32,
    pub next: Option<GeometryId>,
}

pub enum GeometryType {
    Primitive,
    Obstruction,
    Insulation,
}

// 各种几何体的参数结构
pub struct Pyramid {
    pub bottom: [f32; 2],
    pub top: [f32; 2],
    pub offset: [f32; 2],
    pub height: f32,
}

pub struct Cylinder {
    pub radius: f32,
    pub height: f32,
}

// ... 其他几何体类型
```

#### Arena 分配器 (`store/arena.rs`)

```rust
pub struct Arena {
    chunks: Vec<Vec<u8>>,
    current_chunk: usize,
    current_offset: usize,
    chunk_size: usize,
}

impl Arena {
    pub fn new(chunk_size: usize) -> Self;
    pub fn alloc<T>(&mut self, value: T) -> &mut T;
    pub fn alloc_slice<T>(&mut self, slice: &[T]) -> &mut [T];
}
```

#### 字符串驻留 (`store/strings.rs`)

```rust
pub struct StringInterner {
    strings: HashMap<String, StringId>,
    storage: Vec<String>,
}

impl StringInterner {
    pub fn new() -> Self;
    pub fn intern(&mut self, s: &str) -> StringId;
    pub fn get(&self, id: StringId) -> &str;
}

pub struct StringId(u32);
```

#### Store 主结构 (`store/mod.rs`)

```rust
pub struct Store {
    nodes: Arena,
    geometries: Arena,
    strings: StringInterner,
    roots: Vec<NodeId>,
    node_count: usize,
    geometry_count: usize,
}

impl Store {
    pub fn new() -> Self;
    pub fn new_node(&mut self, parent: Option<NodeId>, kind: NodeKind) -> NodeId;
    pub fn new_geometry(&mut self, parent: NodeId, kind: GeometryKind) -> GeometryId;
    pub fn get_node(&self, id: NodeId) -> &Node;
    pub fn get_geometry(&self, id: GeometryId) -> &Geometry;
    pub fn find_root_group(&self, name: &str) -> Option<NodeId>;
}
```

### 3. Math 模块

#### 包围盒 (`math/bbox.rs`)

```rust
pub struct BBox3 {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl BBox3 {
    pub fn new() -> Self;
    pub fn from_min_max(min: glam::Vec3, max: glam::Vec3) -> Self;
    pub fn transform(&self, transform: &glam::Affine3A) -> Self;
    pub fn union(&self, other: &Self) -> Self;
    pub fn contains(&self, point: glam::Vec3) -> bool;
}
```

### 4. Visitor 模块

#### 访问者 trait (`visitor/mod.rs`)

```rust
pub trait Visitor {
    fn visit_node(&mut self, node: &Node, store: &Store);
    fn visit_geometry(&mut self, geometry: &Geometry, store: &Store);
}

pub fn traverse<V: Visitor>(store: &Store, visitor: &mut V);
```

## Data Models

### ID 类型

使用新类型模式确保类型安全：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometryId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorId(u32);
```

### 内存布局

- **Node**: 使用 Arena 分配，通过 ID 引用
- **Geometry**: 使用 Arena 分配，通过 ID 引用
- **String**: 使用字符串驻留，通过 StringId 引用
- **场景图**: 使用链表结构（next, first_child, last_child）

### 数据流

1. **文件读取**: 使用 `memmap2` crate 进行内存映射
2. **解析**: nom 解析器从字节切片中提取数据
3. **存储**: 解析的数据存入 Store 的 Arena
4. **访问**: 通过 ID 访问节点和几何体
5. **遍历**: 使用 Visitor 模式遍历场景图


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Command-line interface compatibility

*For any* valid command-line argument combination from the C++ version, the Rust version should accept and parse it with the same semantics
**Validates: Requirements 1.4**

### Property 2: Block type recognition

*For any* valid RVM file containing HEAD, MODL, CNTB, CNTE, PRIM, OBST, INSU, COLR, or END blocks, the parser should correctly identify each block type
**Validates: Requirements 2.2**

### Property 3: Geometry type support

*For any* of the 11 geometry types (Pyramid, Box, RectangularTorus, CircularTorus, EllipticalDish, SphericalDish, Snout, Cylinder, Sphere, Line, FacetGroup), the parser should correctly parse and store the geometry data
**Validates: Requirements 2.3**

### Property 4: Scene graph hierarchy preservation

*For any* valid RVM file with nested groups, the parsed scene graph should preserve all parent-child relationships and sibling ordering
**Validates: Requirements 2.4**

### Property 5: Parse error reporting

*For any* invalid RVM input, the parser should return an error with information about the failure location
**Validates: Requirements 2.5**

### Property 6: Attribute file parsing

*For any* valid attribute file with NEW/END blocks and key-value pairs, the parser should correctly extract all attributes and associate them with the correct nodes
**Validates: Requirements 3.1**

### Property 7: Attribute syntax handling

*For any* attribute file containing NEW, END, and := syntax, the parser should correctly recognize and process each syntax element
**Validates: Requirements 3.2**

### Property 8: Indentation hierarchy mapping

*For any* attribute file with varying indentation levels, the parser should correctly map indentation to scene graph hierarchy depth
**Validates: Requirements 3.3**

### Property 9: Quote removal in attribute values

*For any* attribute value enclosed in single quotes, the parser should store the value without the surrounding quotes
**Validates: Requirements 3.4**

### Property 10: Multi-attribute line parsing

*For any* attribute line containing multiple key-value pairs separated by &end&, the parser should correctly extract all pairs
**Validates: Requirements 3.5**

### Property 11: Bounding box transformation

*For any* geometry with a local bounding box and transformation matrix, the world-space bounding box should be correctly computed using glam operations
**Validates: Requirements 4.3**

### Property 12: String parsing correctness

*For any* valid RVM string field (length-prefixed, null-padded), the parser should extract the correct string content
**Validates: Requirements 5.4**

### Property 13: Parse error context

*For any* parsing failure, the error should include contextual information about what was expected and what was found
**Validates: Requirements 5.5**

### Property 14: String interning deduplication

*For any* two identical strings stored in the system, they should return the same StringId
**Validates: Requirements 6.2**

### Property 15: Scene graph relationship consistency

*For any* node in the scene graph, if it has a first_child, that child's parent should be the original node, and all siblings should be reachable via next pointers
**Validates: Requirements 7.3**

### Property 16: Color representation support

*For any* color defined either by material ID or RGB values, the system should correctly store and retrieve both representations
**Validates: Requirements 7.5**

### Property 17: File path argument parsing

*For any* valid file path provided as a command-line argument, the system should correctly identify it as an RVM or attribute file based on extension
**Validates: Requirements 9.1**

### Property 18: Invalid argument error handling

*For any* invalid command-line argument, the system should display an error message and exit with a non-zero code
**Validates: Requirements 9.3**

### Property 19: Statistics output completeness

*For any* successfully parsed RVM file, the system should output statistics including node count, geometry count, and geometry type breakdown
**Validates: Requirements 9.4**

### Property 20: Parse failure exit code

*For any* file that fails to parse, the system should exit with a non-zero exit code
**Validates: Requirements 9.5**

## Error Handling

### 解析错误

使用 Rust 的 `Result` 类型和自定义错误类型：

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid chunk header at offset {offset:#x}: expected {expected}, found {found}")]
    InvalidChunkHeader {
        offset: usize,
        expected: String,
        found: String,
    },
    
    #[error("Unexpected end of file at offset {offset:#x}")]
    UnexpectedEof { offset: usize },
    
    #[error("Invalid geometry kind: {kind}")]
    InvalidGeometryKind { kind: u32 },
    
    #[error("Nom parsing error: {0}")]
    NomError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 错误传播

- nom 解析器返回 `IResult<&[u8], T>`
- 高层函数将 nom 错误转换为 `ParseError`
- 使用 `?` 运算符进行错误传播
- 在 main 函数中捕获并显示错误

### 错误恢复

- 解析错误时立即返回，不尝试恢复
- 提供详细的错误位置和上下文
- 记录部分解析的数据用于调试

## Testing Strategy

### 单元测试

**解析器测试**:
- 测试每个 nom 组合子的正确性
- 使用手工构造的小型二进制数据
- 验证解析结果与预期匹配
- 测试边界条件（空输入、最小/最大值）

**数据结构测试**:
- 测试 Arena 分配器的分配和访问
- 测试字符串驻留的去重功能
- 测试场景图的构建和遍历
- 测试包围盒计算的正确性

**数学运算测试**:
- 测试包围盒变换
- 测试矩阵和向量运算
- 使用已知输入和输出验证

### 属性测试

使用 `proptest` crate 进行基于属性的测试：

**Property 1: String interning idempotence**
- 生成随机字符串
- 验证多次 intern 返回相同 ID
- 验证通过 ID 获取的字符串与原始字符串相同

**Property 2: Bounding box transformation**
- 生成随机包围盒和变换矩阵
- 验证变换后的包围盒包含所有变换后的角点

**Property 3: Scene graph traversal**
- 生成随机场景图结构
- 验证遍历访问所有节点恰好一次
- 验证父子关系一致性

**Property 4: Parse-serialize round trip** (未来扩展)
- 生成随机 RVM 数据结构
- 序列化为二进制
- 解析回数据结构
- 验证结果与原始数据等价

### 集成测试

**与 C++ 版本对比**:
- 使用相同的测试 RVM 文件
- 比较解析后的统计信息（节点数、几何体数）
- 比较导出的数据（如果实现导出功能）
- 验证错误处理行为一致

**真实文件测试**:
- 使用实际的 AVEVA PDMS RVM 文件
- 验证能够成功解析
- 验证性能满足要求
- 验证内存使用合理

### 测试数据

- 创建最小化的测试 RVM 文件
- 包含所有几何类型的示例
- 包含嵌套层次结构的示例
- 包含各种边界情况的示例

### 性能测试

- 使用 `criterion` crate 进行基准测试
- 测试解析速度
- 测试内存使用
- 与 C++ 版本比较性能

## Implementation Notes

### 使用 glam 的注意事项

1. **对齐要求**: `glam::Mat3A` 和 `glam::Affine3A` 有 16 字节对齐要求
2. **SIMD 优化**: glam 在支持的平台上使用 SIMD 指令
3. **坐标系**: 注意 RVM 使用的坐标系约定

### 使用 nom 的注意事项

1. **零拷贝**: 尽可能使用 `&[u8]` 和 `&str` 避免分配
2. **错误处理**: 使用 `nom::error::context` 添加错误上下文
3. **组合子选择**: 使用 `complete` 版本的组合子确保完整解析
4. **性能**: 避免过度使用 `many0` 等可能导致回溯的组合子

### 内存管理策略

1. **Arena 分配**: 用于节点和几何体，批量释放
2. **字符串驻留**: 减少字符串重复存储
3. **引用计数**: 仅在必要时使用 `Rc` 或 `Arc`
4. **内存映射**: 使用 `memmap2` 避免加载整个文件

### 与 C++ 版本的差异

1. **所有权模型**: Rust 的所有权系统替代 C++ 的手动内存管理
2. **错误处理**: 使用 `Result` 替代返回码和错误字符串
3. **类型安全**: 使用枚举和新类型模式增强类型安全
4. **迭代器**: 使用 Rust 迭代器替代 C++ 的指针遍历

### 未来扩展

本设计为以下功能预留了扩展空间：

1. **导出功能**: 通过实现 Visitor trait 添加 OBJ、GLTF、JSON 导出
2. **曲面细分**: 添加 Tessellator 模块将基本几何体转换为网格
3. **几何连接**: 添加连接检测和对齐功能
4. **层次扁平化**: 添加基于正则表达式的层次过滤
5. **并行处理**: 使用 rayon 并行处理独立的几何体

## Dependencies

```toml
[dependencies]
glam = "0.33"
nom = "8.0.0"
memmap2 = "0.9"
thiserror = "2.0"

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"
```

## Build and Development

### 构建命令

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 运行基准测试
cargo bench

# 检查代码
cargo clippy

# 格式化代码
cargo fmt
```

### 开发流程

1. 实现核心数据结构（Store, Node, Geometry）
2. 实现基本的 nom 组合子（数字、字符串）
3. 实现 RVM 块解析器（HEAD, MODL, CNTB, PRIM）
4. 实现属性文件解析器
5. 实现命令行接口
6. 添加测试和文档
7. 性能优化和与 C++ 版本对比

### 代码风格

- 遵循 Rust 官方风格指南
- 使用 `cargo fmt` 自动格式化
- 使用 `cargo clippy` 检查常见问题
- 为公共 API 编写文档注释
- 使用有意义的变量和函数名
