# Requirements Document

## Introduction

本项目旨在为 Rust 版本的 RVM 解析器添加导出功能，支持将解析后的 RVM 数据导出为多种常用的 3D 格式，包括 OBJ、JSON 和 GLTF/GLB。这些导出功能将使用访问者模式实现，便于扩展和维护。

## Glossary

- **OBJ Format**: Wavefront OBJ 文件格式，一种广泛使用的 3D 模型文件格式
- **MTL File**: Material Template Library，OBJ 格式的材质文件
- **JSON Export**: 将场景图结构导出为 JSON 格式，包含层次结构和属性
- **GLTF**: GL Transmission Format，Khronos 组织的 3D 场景传输格式
- **GLB**: GLTF 的二进制容器格式
- **Triangulation**: 曲面细分，将基本几何体转换为三角网格
- **Visitor Pattern**: 访问者模式，用于遍历场景图并执行操作
- **serde**: Rust 的序列化/反序列化框架
- **serde_json**: serde 的 JSON 实现

## Requirements

### Requirement 1

**User Story:** 作为用户，我希望能够将 RVM 数据导出为 OBJ 格式，以便在其他 3D 软件中使用。

#### Acceptance Criteria

1. WHEN 用户指定 OBJ 导出路径时 THEN 系统 SHALL 创建 .obj 和 .mtl 文件
2. WHEN 导出几何体时 THEN 系统 SHALL 将三角网格写入 OBJ 文件
3. WHEN 导出材质时 THEN 系统 SHALL 将颜色和透明度信息写入 MTL 文件
4. WHEN 导出场景图时 THEN 系统 SHALL 使用对象名称（o）标记不同的组
5. WHEN 导出完成时 THEN 系统 SHALL 输出顶点、法线和面索引

### Requirement 2

**User Story:** 作为用户，我希望能够将场景图导出为 JSON 格式，以便进行数据分析和处理。

#### Acceptance Criteria

1. WHEN 用户指定 JSON 导出路径时 THEN 系统 SHALL 创建 JSON 文件
2. WHEN 导出节点时 THEN 系统 SHALL 保留完整的层次结构
3. WHEN 导出属性时 THEN 系统 SHALL 包含所有键值对
4. WHEN 导出包围盒时 THEN 系统 SHALL 包含世界空间的包围盒数据
5. WHEN 导出完成时 THEN 系统 SHALL 生成格式化的 JSON 输出

### Requirement 3

**User Story:** 作为用户，我希望能够将 RVM 数据导出为 GLTF/GLB 格式，以便在现代 3D 引擎和查看器中使用。

#### Acceptance Criteria

1. WHEN 用户指定 .gltf 扩展名时 THEN 系统 SHALL 导出为 GLTF 格式
2. WHEN 用户指定 .glb 扩展名时 THEN 系统 SHALL 导出为 GLB 二进制格式
3. WHEN 导出几何体时 THEN 系统 SHALL 创建 GLTF 网格和访问器
4. WHEN 导出材质时 THEN 系统 SHALL 创建 PBR 材质定义
5. WHEN 导出场景图时 THEN 系统 SHALL 保留节点层次结构和变换矩阵

### Requirement 4

**User Story:** 作为用户，我希望系统能够对基本几何体进行曲面细分，以便导出为三角网格。

#### Acceptance Criteria

1. WHEN 几何体需要导出时 THEN 系统 SHALL 将基本几何体转换为三角网格
2. WHEN 细分圆柱体时 THEN 系统 SHALL 生成指定精度的三角网格
3. WHEN 细分球体时 THEN 系统 SHALL 生成均匀分布的三角网格
4. WHEN 细分复杂几何体时 THEN 系统 SHALL 保持合理的三角形数量
5. WHEN 细分完成时 THEN 系统 SHALL 计算正确的法线向量

### Requirement 5

**User Story:** 作为用户，我希望导出功能支持配置选项，以便控制导出行为。

#### Acceptance Criteria

1. WHEN 用户启用中心化选项时 THEN 系统 SHALL 将模型中心移至原点
2. WHEN 用户启用坐标系转换时 THEN 系统 SHALL 将 Z 轴转换为 Y 轴
3. WHEN 用户启用属性导出时 THEN 系统 SHALL 在导出中包含所有属性
4. WHEN 用户启用几何合并时 THEN 系统 SHALL 合并相同材质的几何体
5. WHEN 用户指定导出选项时 THEN 系统 SHALL 应用这些选项到导出过程

### Requirement 6

**User Story:** 作为用户，我希望导出过程能够处理大型模型，以便导出复杂的工业模型。

#### Acceptance Criteria

1. WHEN 导出大型模型时 THEN 系统 SHALL 使用流式写入避免内存溢出
2. WHEN 处理大量几何体时 THEN 系统 SHALL 批量处理以提高性能
3. WHEN 导出失败时 THEN 系统 SHALL 提供清晰的错误信息
4. WHEN 导出进度时 THEN 系统 SHALL 可选地报告进度信息
5. WHEN 导出完成时 THEN 系统 SHALL 验证输出文件的完整性

### Requirement 7

**User Story:** 作为用户，我希望命令行工具支持导出功能，以便直接使用。

#### Acceptance Criteria

1. WHEN 用户提供 --export-obj 参数时 THEN 系统 SHALL 导出为 OBJ 格式
2. WHEN 用户提供 --export-json 参数时 THEN 系统 SHALL 导出为 JSON 格式
3. WHEN 用户提供 --export-gltf 参数时 THEN 系统 SHALL 导出为 GLTF/GLB 格式
4. WHEN 用户提供多个导出选项时 THEN 系统 SHALL 执行所有导出操作
5. WHEN 导出参数无效时 THEN 系统 SHALL 显示错误信息并退出

### Requirement 8

**User Story:** 作为开发者，我希望导出功能使用访问者模式，以便易于扩展新的导出格式。

#### Acceptance Criteria

1. WHEN 实现导出器时 THEN 系统 SHALL 实现 Visitor trait
2. WHEN 遍历场景图时 THEN 系统 SHALL 调用访问者的回调方法
3. WHEN 添加新导出格式时 THEN 系统 SHALL 只需实现新的访问者
4. WHEN 访问节点时 THEN 系统 SHALL 提供节点类型和数据的访问
5. WHEN 访问几何体时 THEN 系统 SHALL 提供几何体类型和三角网格的访问

### Requirement 9

**User Story:** 作为用户，我希望 OBJ 导出支持材质和颜色，以便保留视觉外观。

#### Acceptance Criteria

1. WHEN 导出几何体时 THEN 系统 SHALL 为每个唯一颜色创建材质
2. WHEN 写入 MTL 文件时 THEN 系统 SHALL 包含环境光、漫反射和镜面反射
3. WHEN 几何体有透明度时 THEN 系统 SHALL 在材质中设置 d 参数
4. WHEN 使用材质时 THEN 系统 SHALL 在 OBJ 文件中使用 usemtl 指令
5. WHEN 导出完成时 THEN 系统 SHALL 确保所有引用的材质都已定义

### Requirement 10

**User Story:** 作为用户，我希望 GLTF 导出符合规范，以便在标准查看器中正确显示。

#### Acceptance Criteria

1. WHEN 导出 GLTF 时 THEN 系统 SHALL 生成符合 GLTF 2.0 规范的文件
2. WHEN 创建访问器时 THEN 系统 SHALL 正确设置 min 和 max 值
3. WHEN 创建缓冲区视图时 THEN 系统 SHALL 正确对齐数据
4. WHEN 使用 GLB 格式时 THEN 系统 SHALL 正确写入二进制块
5. WHEN 导出完成时 THEN 系统 SHALL 生成可被标准查看器加载的文件
