# Implementation Plan

- [x] 1. 完整实现几何处理功能（连接检测和对齐）


  - 扩展 Geometry 结构添加 sample_start_angle 和 connections 字段
  - 创建 store/connection.rs 定义 Connection 数据结构
  - 创建 processing 模块结构（mod.rs, anchor.rs, connection.rs, alignment.rs）
  - 实现锚点生成（anchor.rs）：
    - 定义 Anchor 和 ConnectionFlags 数据结构
    - 实现所有几何类型的锚点生成函数（Pyramid, Box, Cylinder, CircularTorus, RectangularTorus, Snout, EllipticalDish, SphericalDish）
    - 实现局部到世界空间的变换
    - 实现法线归一化
  - 实现连接检测器（connection.rs）：
    - 创建 ConnectionDetector 结构体
    - 实现锚点生成和收集
    - 实现空间排序（按 X 坐标）
    - 实现匹配算法（距离和法线检查）
    - 实现连接创建和存储
    - 实现统计信息收集
  - 实现对齐处理器（alignment.rs）：
    - 创建 AlignmentProcessor 结构体
    - 实现连通分量识别（BFS）
    - 实现圆柱对齐算法
    - 实现圆环对齐算法
    - 实现 Snout 对齐算法
    - 实现 Dish 对齐算法
    - 实现对齐传播逻辑
  - 实现处理模块入口（processing/mod.rs）：
    - 定义 ProcessingError 类型
    - 导出公共接口
    - 实现便捷函数
  - 集成到主程序：
    - 在解析完成后调用连接检测
    - 在连接检测后调用对齐处理
    - 添加命令行选项控制处理行为
    - 输出处理统计信息
  - 更新 lib.rs 导出 processing 模块
  - 运行 cargo clippy 和 cargo fmt 确保代码质量
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 8.5, 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4, 10.5_
