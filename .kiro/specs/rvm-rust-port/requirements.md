# Requirements Document

## Introduction

本项目旨在将现有的 C++ rvmparser 工具移植到 Rust 语言，使用 glam 0.33 进行线性代数运算，使用 nom 0.8 进行二进制解析。移植后的工具应保持与原始 C++ 版本相同的功能，能够读取和处理 AVEVA PDMS RVM 文件及其属性文件，并支持多种导出格式。

## Glossary

- **RVM File**: AVEVA PDMS 二进制几何文件，包含 3D 模型的几何信息
- **Attribute File**: 与 RVM 文件配套的文本属性文件（.txt 或 .att 后缀）
- **Parser**: 解析器，负责读取和解析 RVM 和属性文件的组件
- **Store**: 数据存储结构，用于保存解析后的场景图和几何数据
- **Geometry**: 几何体，包括金字塔、盒子、圆柱、球体等基本形状
- **Node**: 场景图节点，表示文件、模型或组的层次结构
- **Tessellator**: 曲面细分器，将基本几何体转换为三角网格
- **glam**: Rust 的数学库，用于向量、矩阵等线性代数运算
- **nom**: Rust 的解析器组合子库（版本 8.0.0），用于构建二进制和文本解析器

## Requirements

### Requirement 1

**User Story:** 作为开发者，我希望使用 Rust 重新实现 RVM 解析器，以便获得内存安全和更好的性能保证。

#### Acceptance Criteria

1. WHEN 系统初始化时 THEN 系统 SHALL 使用 glam 0.33 作为线性代数库
2. WHEN 系统初始化时 THEN 系统 SHALL 使用 nom 8.0.0 作为解析器框架
3. WHEN 系统编译时 THEN 系统 SHALL 不产生任何 unsafe 代码警告（除非明确需要且有文档说明）
4. WHEN 系统运行时 THEN 系统 SHALL 提供与 C++ 版本相同的命令行接口

### Requirement 2

**User Story:** 作为用户，我希望能够解析 RVM 二进制文件，以便提取 3D 几何信息。

#### Acceptance Criteria

1. WHEN 用户提供 .rvm 文件路径时 THEN 系统 SHALL 使用内存映射方式读取文件内容
2. WHEN 解析 RVM 文件时 THEN 系统 SHALL 正确识别和解析 HEAD、MODL、CNTB、CNTE、PRIM、OBST、INSU、COLR、END 等块类型
3. WHEN 解析几何体时 THEN 系统 SHALL 支持所有 11 种几何类型（金字塔、盒子、矩形环、圆环、椭圆碟、球形碟、锥台、圆柱、球体、线、面组）
4. WHEN 解析完成时 THEN 系统 SHALL 构建完整的场景图层次结构
5. WHEN 解析失败时 THEN 系统 SHALL 返回清晰的错误信息，包括失败位置和原因

### Requirement 3

**User Story:** 作为用户，我希望能够解析属性文件，以便将元数据关联到几何对象。

#### Acceptance Criteria

1. WHEN 用户提供 .txt 或 .att 文件路径时 THEN 系统 SHALL 解析属性文件内容
2. WHEN 解析属性文件时 THEN 系统 SHALL 正确处理 NEW、END 和属性赋值（:=）语法
3. WHEN 解析属性时 THEN 系统 SHALL 正确处理缩进层次以匹配场景图结构
4. WHEN 属性值包含引号时 THEN 系统 SHALL 正确去除引号
5. WHEN 属性包含多个键值对时 THEN 系统 SHALL 正确处理 &end& 分隔符

### Requirement 4

**User Story:** 作为用户，我希望使用 glam 进行所有数学运算，以便获得高性能的向量和矩阵操作。

#### Acceptance Criteria

1. WHEN 系统处理变换矩阵时 THEN 系统 SHALL 使用 glam::Mat3A 或 glam::Affine3A 表示 3x4 变换矩阵
2. WHEN 系统处理向量时 THEN 系统 SHALL 使用 glam::Vec3 表示三维向量
3. WHEN 系统计算包围盒时 THEN 系统 SHALL 使用 glam 的向量运算进行变换
4. WHEN 系统进行矩阵乘法时 THEN 系统 SHALL 使用 glam 提供的运算符重载

### Requirement 5

**User Story:** 作为用户，我希望使用 nom 构建解析器，以便获得组合式、可测试的解析逻辑。

#### Acceptance Criteria

1. WHEN 解析二进制数据时 THEN 系统 SHALL 使用 nom 的组合子构建解析器
2. WHEN 解析大端序整数时 THEN 系统 SHALL 使用 nom::number::complete::be_u32 等函数
3. WHEN 解析浮点数时 THEN 系统 SHALL 使用 nom::number::complete::be_f32 函数
4. WHEN 解析字符串时 THEN 系统 SHALL 实现自定义的字符串解析组合子
5. WHEN 解析失败时 THEN 系统 SHALL 利用 nom 的错误处理机制提供上下文信息

### Requirement 6

**User Story:** 作为用户，我希望系统能够高效管理内存，以便处理大型 RVM 文件。

#### Acceptance Criteria

1. WHEN 系统分配内存时 THEN 系统 SHALL 使用 Rust 的所有权系统避免内存泄漏
2. WHEN 系统存储字符串时 THEN 系统 SHALL 实现字符串驻留机制以减少重复
3. WHEN 系统构建场景图时 THEN 系统 SHALL 使用 Arena 分配器或类似机制进行批量分配
4. WHEN 系统处理大文件时 THEN 系统 SHALL 使用内存映射避免一次性加载整个文件

### Requirement 7

**User Story:** 作为用户，我希望系统提供清晰的数据结构，以便后续处理和导出。

#### Acceptance Criteria

1. WHEN 系统存储节点时 THEN 系统 SHALL 使用枚举区分 File、Model、Group 三种节点类型
2. WHEN 系统存储几何体时 THEN 系统 SHALL 使用枚举区分 11 种几何类型
3. WHEN 系统存储场景图时 THEN 系统 SHALL 维护父子关系和兄弟链表
4. WHEN 系统存储属性时 THEN 系统 SHALL 使用键值对结构
5. WHEN 系统存储颜色时 THEN 系统 SHALL 支持材质 ID 和 RGB 颜色

### Requirement 8

**User Story:** 作为开发者，我希望代码具有良好的可测试性，以便验证移植的正确性。

#### Acceptance Criteria

1. WHEN 实现解析器时 THEN 系统 SHALL 将解析逻辑与 I/O 分离
2. WHEN 实现数据结构时 THEN 系统 SHALL 提供构造函数和访问器方法
3. WHEN 实现转换时 THEN 系统 SHALL 提供纯函数接口
4. WHEN 编写测试时 THEN 系统 SHALL 能够使用小型测试数据验证解析器
5. WHEN 编写测试时 THEN 系统 SHALL 能够比较 Rust 版本和 C++ 版本的输出

### Requirement 9

**User Story:** 作为用户，我希望系统提供命令行工具，以便直接使用解析功能。

#### Acceptance Criteria

1. WHEN 用户运行程序时 THEN 系统 SHALL 接受文件路径作为参数
2. WHEN 用户提供 --help 参数时 THEN 系统 SHALL 显示使用说明
3. WHEN 用户提供无效参数时 THEN 系统 SHALL 显示错误信息并退出
4. WHEN 解析成功时 THEN 系统 SHALL 输出统计信息
5. WHEN 解析失败时 THEN 系统 SHALL 返回非零退出码

### Requirement 10

**User Story:** 作为开发者，我希望代码结构清晰，以便后续扩展导出功能。

#### Acceptance Criteria

1. WHEN 组织代码时 THEN 系统 SHALL 将解析器、数据结构、工具函数分离到不同模块
2. WHEN 设计 API 时 THEN 系统 SHALL 提供访问者模式接口用于遍历场景图
3. WHEN 设计数据结构时 THEN 系统 SHALL 使用 trait 定义通用行为
4. WHEN 实现功能时 THEN 系统 SHALL 保持模块间的低耦合
5. WHEN 添加新功能时 THEN 系统 SHALL 不需要修改核心数据结构
