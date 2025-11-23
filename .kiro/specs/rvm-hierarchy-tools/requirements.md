# Requirements Document

## Introduction

本项目旨在为 Rust 版本的 RVM 解析器添加层次结构处理工具，包括基于正则表达式的层次扁平化、基于标签列表的组保留/合并、以及组丢弃功能。这些工具可以简化复杂的场景图结构，便于后续处理和导出。

## Glossary

- **Flatten**: 扁平化，简化层次结构，移除不需要的中间节点
- **Regex Flatten**: 基于正则表达式的扁平化，保留名称匹配正则表达式的节点
- **Keep Groups**: 保留组，指定要保留的组名称列表
- **Discard Groups**: 丢弃组，指定要删除的组名称列表
- **Merge**: 合并，将子节点的几何体和属性移动到父节点
- **Tag**: 标签，组的名称标识符
- **Hierarchy**: 层次结构，场景图的树形结构

## Requirements

### Requirement 1

**User Story:** 作为用户，我希望能够使用正则表达式过滤场景图，以便只保留感兴趣的节点。

#### Acceptance Criteria

1. WHEN 用户提供正则表达式时 THEN 系统 SHALL 编译并验证正则表达式
2. WHEN 节点名称匹配正则表达式时 THEN 系统 SHALL 保留该节点
3. WHEN 节点名称不匹配时 THEN 系统 SHALL 将其几何体和属性移至最近的匹配祖先
4. WHEN 扁平化完成时 THEN 系统 SHALL 报告节点和几何体数量的变化
5. WHEN 正则表达式无效时 THEN 系统 SHALL 返回错误信息

### Requirement 2

**User Story:** 作为用户，我希望能够指定要保留的组列表，以便只导出特定的组。

#### Acceptance Criteria

1. WHEN 用户提供组名列表文件时 THEN 系统 SHALL 读取并解析文件
2. WHEN 组在保留列表中时 THEN 系统 SHALL 保留该组及其祖先
3. WHEN 组不在列表中且无子组在列表中时 THEN 系统 SHALL 将其合并到父组
4. WHEN 保留处理完成时 THEN 系统 SHALL 创建新的简化场景图
5. WHEN 列表文件不存在时 THEN 系统 SHALL 返回错误信息

### Requirement 3

**User Story:** 作为用户，我希望能够指定要丢弃的组列表，以便移除不需要的部分。

#### Acceptance Criteria

1. WHEN 用户提供丢弃列表文件时 THEN 系统 SHALL 读取并解析文件
2. WHEN 组在丢弃列表中时 THEN 系统 SHALL 删除该组及其所有子节点
3. WHEN 丢弃处理完成时 THEN 系统 SHALL 更新场景图结构
4. WHEN 丢弃处理完成时 THEN 系统 SHALL 报告丢弃的组数量
5. WHEN 列表文件格式错误时 THEN 系统 SHALL 返回错误信息

### Requirement 4

**User Story:** 作为用户，我希望扁平化操作保留几何体和属性，以便不丢失数据。

#### Acceptance Criteria

1. WHEN 节点被移除时 THEN 系统 SHALL 将其几何体移至保留的祖先
2. WHEN 节点被移除时 THEN 系统 SHALL 将其属性移至保留的祖先
3. WHEN 移动几何体时 THEN 系统 SHALL 保持几何体的变换矩阵
4. WHEN 移动属性时 THEN 系统 SHALL 保持属性的键值对
5. WHEN 扁平化完成时 THEN 系统 SHALL 验证所有几何体都已保留

### Requirement 5

**User Story:** 作为用户，我希望层次工具支持递归处理，以便处理深层嵌套的结构。

#### Acceptance Criteria

1. WHEN 处理嵌套组时 THEN 系统 SHALL 递归遍历所有子节点
2. WHEN 子节点需要保留时 THEN 系统 SHALL 保留从根到该节点的路径
3. WHEN 子节点需要移除时 THEN 系统 SHALL 递归处理其子节点
4. WHEN 递归完成时 THEN 系统 SHALL 确保层次结构一致性
5. WHEN 遇到循环引用时 THEN 系统 SHALL 检测并报告错误

### Requirement 6

**User Story:** 作为用户，我希望保留操作创建新的 Store，以便不修改原始数据。

#### Acceptance Criteria

1. WHEN 执行保留操作时 THEN 系统 SHALL 创建新的 Store 实例
2. WHEN 复制节点时 THEN 系统 SHALL 深拷贝节点数据
3. WHEN 复制几何体时 THEN 系统 SHALL 深拷贝几何体数据
4. WHEN 复制完成时 THEN 系统 SHALL 返回新的 Store
5. WHEN 原始 Store 时 THEN 系统 SHALL 保持不变

### Requirement 7

**User Story:** 作为用户，我希望层次工具支持标签文件格式，以便从文本文件读取配置。

#### Acceptance Criteria

1. WHEN 读取标签文件时 THEN 系统 SHALL 支持每行一个标签的格式
2. WHEN 标签包含缩进时 THEN 系统 SHALL 忽略前导空白
3. WHEN 标签包含尾随空白时 THEN 系统 SHALL 忽略尾随空白
4. WHEN 遇到空行时 THEN 系统 SHALL 跳过空行
5. WHEN 文件编码为 UTF-8 时 THEN 系统 SHALL 正确解析

### Requirement 8

**User Story:** 作为用户，我希望扁平化操作保留最低层级的组，以便确保几何体有容器。

#### Acceptance Criteria

1. WHEN 所有组都不匹配时 THEN 系统 SHALL 至少保留前两层组
2. WHEN 第一层组不匹配时 THEN 系统 SHALL 强制保留以容纳几何体
3. WHEN 几何体需要容器时 THEN 系统 SHALL 创建或保留父组
4. WHEN 保留最低层级时 THEN 系统 SHALL 标记为特殊保留
5. WHEN 导出时 THEN 系统 SHALL 确保几何体有有效的父节点

### Requirement 9

**User Story:** 作为用户，我希望命令行工具支持层次处理选项，以便直接使用。

#### Acceptance Criteria

1. WHEN 用户提供 --keep-regex 参数时 THEN 系统 SHALL 执行正则表达式扁平化
2. WHEN 用户提供 --keep-groups 参数时 THEN 系统 SHALL 执行保留操作
3. WHEN 用户提供 --discard-groups 参数时 THEN 系统 SHALL 执行丢弃操作
4. WHEN 用户提供多个选项时 THEN 系统 SHALL 按顺序执行操作
5. WHEN 参数无效时 THEN 系统 SHALL 显示错误信息并退出

### Requirement 10

**User Story:** 作为开发者，我希望层次工具模块化，以便独立测试和维护。

#### Acceptance Criteria

1. WHEN 实现正则扁平化时 THEN 系统 SHALL 将其作为独立函数
2. WHEN 实现保留操作时 THEN 系统 SHALL 将其作为独立结构体
3. WHEN 实现丢弃操作时 THEN 系统 SHALL 将其作为独立函数
4. WHEN 添加新操作时 THEN 系统 SHALL 不需要修改现有操作
5. WHEN 测试时 THEN 系统 SHALL 能够独立测试每个操作
