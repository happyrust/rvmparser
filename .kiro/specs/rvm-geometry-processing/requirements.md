# Requirements Document

## Introduction

本项目旨在为 Rust 版本的 RVM 解析器添加几何处理功能，包括相邻几何体的连接检测和圆周采样点对齐。这些功能可以改善导出网格的质量，避免在连接处产生不必要的内部封盖，并确保相邻几何体的采样点对齐。

## Glossary

- **Connection**: 连接，表示两个几何体之间的相邻关系
- **Anchor**: 锚点，几何体表面的连接点，包含位置和法线方向
- **Circular Side**: 圆形侧面，如圆柱、圆环的端面
- **Rectangular Side**: 矩形侧面，如盒子、金字塔的面
- **Alignment**: 对齐，调整相邻几何体的采样起始角度使其匹配
- **Sample Start Angle**: 采样起始角度，圆形几何体开始细分的角度
- **Connected Component**: 连通分量，通过连接关系相连的几何体集合

## Requirements

### Requirement 1

**User Story:** 作为用户，我希望系统能够检测相邻几何体的连接关系，以便在导出时优化网格质量。

#### Acceptance Criteria

1. WHEN 系统处理场景图时 THEN 系统 SHALL 识别所有几何体的潜在连接面
2. WHEN 两个几何体的连接面接近时 THEN 系统 SHALL 创建连接关系
3. WHEN 连接面法线相反时 THEN 系统 SHALL 确认连接有效
4. WHEN 连接检测完成时 THEN 系统 SHALL 报告匹配的锚点数量
5. WHEN 连接关系建立时 THEN 系统 SHALL 存储连接的几何体和偏移量

### Requirement 2

**User Story:** 作为用户，我希望系统能够为不同几何类型生成正确的锚点，以便进行连接检测。

#### Acceptance Criteria

1. WHEN 处理金字塔时 THEN 系统 SHALL 生成 6 个锚点（4 个侧面 + 2 个底面）
2. WHEN 处理盒子时 THEN 系统 SHALL 生成 6 个锚点（6 个面）
3. WHEN 处理圆柱时 THEN 系统 SHALL 生成 2 个锚点（2 个端面）
4. WHEN 处理圆环时 THEN 系统 SHALL 生成 2 个锚点（2 个端面）
5. WHEN 处理球体和线时 THEN 系统 SHALL 不生成锚点

### Requirement 3

**User Story:** 作为用户，我希望系统能够区分圆形和矩形连接面，以便正确处理不同类型的连接。

#### Acceptance Criteria

1. WHEN 锚点属于圆形面时 THEN 系统 SHALL 标记为 HasCircularSide
2. WHEN 锚点属于矩形面时 THEN 系统 SHALL 标记为 HasRectangularSide
3. WHEN 连接包含圆形面时 THEN 系统 SHALL 记录连接标志
4. WHEN 连接仅包含矩形面时 THEN 系统 SHALL 跳过对齐处理
5. WHEN 连接包含至少一个圆形面时 THEN 系统 SHALL 执行对齐处理

### Requirement 4

**User Story:** 作为用户，我希望系统能够对齐相邻几何体的圆周采样点，以便改善网格连接质量。

#### Acceptance Criteria

1. WHEN 两个圆形几何体连接时 THEN 系统 SHALL 计算对齐的采样起始角度
2. WHEN 圆柱连接时 THEN 系统 SHALL 将采样角度投影到端面平面
3. WHEN 圆环连接时 THEN 系统 SHALL 考虑圆环的旋转角度
4. WHEN 对齐完成时 THEN 系统 SHALL 更新几何体的 sampleStartAngle 字段
5. WHEN 对齐传播时 THEN 系统 SHALL 沿连接关系传播对齐信息

### Requirement 5

**User Story:** 作为用户，我希望系统能够识别连通分量，以便独立处理不同的几何体组。

#### Acceptance Criteria

1. WHEN 系统开始对齐时 THEN 系统 SHALL 识别所有连通分量
2. WHEN 处理连通分量时 THEN 系统 SHALL 为每个分量选择任意起始方向
3. WHEN 遍历连通分量时 THEN 系统 SHALL 使用广度优先搜索
4. WHEN 对齐完成时 THEN 系统 SHALL 报告连通分量数量
5. WHEN 几何体已处理时 THEN 系统 SHALL 跳过重复处理

### Requirement 6

**User Story:** 作为用户，我希望连接检测使用空间索引，以便高效处理大型模型。

#### Acceptance Criteria

1. WHEN 锚点数量较多时 THEN 系统 SHALL 使用空间排序优化搜索
2. WHEN 搜索匹配锚点时 THEN 系统 SHALL 按 X 坐标排序
3. WHEN 检查匹配时 THEN 系统 SHALL 使用距离阈值快速剔除
4. WHEN 匹配成功时 THEN 系统 SHALL 标记锚点避免重复匹配
5. WHEN 连接检测完成时 THEN 系统 SHALL 移除已匹配的锚点

### Requirement 7

**User Story:** 作为用户，我希望系统能够处理变换矩阵，以便在世界空间中进行连接检测。

#### Acceptance Criteria

1. WHEN 生成锚点时 THEN 系统 SHALL 将局部坐标转换到世界空间
2. WHEN 计算法线时 THEN 系统 SHALL 应用变换矩阵的旋转部分
3. WHEN 比较位置时 THEN 系统 SHALL 使用世界空间坐标
4. WHEN 计算距离时 THEN 系统 SHALL 使用欧几里得距离
5. WHEN 检查对齐时 THEN 系统 SHALL 使用法线向量的点积

### Requirement 8

**User Story:** 作为用户，我希望系统能够处理特殊几何类型，以便正确生成锚点。

#### Acceptance Criteria

1. WHEN 处理 Snout（锥台）时 THEN 系统 SHALL 考虑剪切变换
2. WHEN 处理 EllipticalDish 时 THEN 系统 SHALL 生成底部锚点
3. WHEN 处理 SphericalDish 时 THEN 系统 SHALL 生成底部锚点
4. WHEN 处理 FacetGroup 时 THEN 系统 SHALL 不生成锚点
5. WHEN 处理 Line 时 THEN 系统 SHALL 不生成锚点

### Requirement 9

**User Story:** 作为用户，我希望系统提供配置选项，以便控制连接检测和对齐行为。

#### Acceptance Criteria

1. WHEN 用户指定距离阈值时 THEN 系统 SHALL 使用该阈值判断连接
2. WHEN 用户指定角度阈值时 THEN 系统 SHALL 使用该阈值判断对齐
3. WHEN 用户禁用连接检测时 THEN 系统 SHALL 跳过连接检测
4. WHEN 用户禁用对齐时 THEN 系统 SHALL 跳过对齐处理
5. WHEN 用户启用调试输出时 THEN 系统 SHALL 输出连接和对齐信息

### Requirement 10

**User Story:** 作为开发者，我希望连接和对齐功能模块化，以便独立测试和维护。

#### Acceptance Criteria

1. WHEN 实现连接检测时 THEN 系统 SHALL 将其作为独立模块
2. WHEN 实现对齐时 THEN 系统 SHALL 将其作为独立模块
3. WHEN 调用连接检测时 THEN 系统 SHALL 不依赖对齐模块
4. WHEN 调用对齐时 THEN 系统 SHALL 依赖连接检测结果
5. WHEN 添加新几何类型时 THEN 系统 SHALL 只需扩展锚点生成逻辑
