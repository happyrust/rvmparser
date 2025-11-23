# Implementation Plan

- [x] 1. 完整实现 RVM 导出功能（OBJ、JSON、GLTF/GLB）




  - 添加 serde 和 serde_json 依赖到 Cargo.toml
  - 创建完整的 export 模块结构（mod.rs, tessellator.rs, obj.rs, json.rs, gltf.rs）
  - 实现曲面细分器（Tessellator）：
    - 定义 Triangulation 数据结构
    - 实现所有几何类型的细分（Cylinder, Sphere, Box, Pyramid, CircularTorus, RectangularTorus, EllipticalDish, SphericalDish, Snout, Line, FacetGroup）
    - 实现法线计算和归一化
  - 实现 OBJ 导出器：
    - 创建 ObjExporter 结构体实现 Visitor trait
    - 实现 OBJ 和 MTL 文件写入
    - 实现材质管理和去重
    - 实现顶点、法线、面索引写入
    - 实现对象分组和层次结构
  - 实现 JSON 导出器：
    - 创建 JsonExporter 结构体实现 Visitor trait
    - 定义 JSON 数据结构（使用 serde）
    - 实现场景图遍历和序列化
    - 实现属性和包围盒导出
  - 实现 GLTF/GLB 导出器：
    - 创建 GltfExporter 结构体实现 Visitor trait
    - 实现缓冲区管理和数据对齐
    - 实现访问器和缓冲区视图创建
    - 实现 PBR 材质创建
    - 实现场景图和网格导出
    - 实现 GLTF 文本格式写入
    - 实现 GLB 二进制格式写入
    - 实现坐标系转换和模型中心化选项
  - 实现导出模块入口：
    - 定义 ExportError 类型
    - 导出公共接口
  - 更新命令行接口：
    - 添加 --export-obj, --export-json, --export-gltf 参数
    - 添加导出选项参数（--center, --rotate-z-to-y, --include-attributes, --merge-geometries）
    - 实现多格式同时导出
    - 更新帮助信息
  - 更新 lib.rs 导出 export 模块
  - 运行 cargo clippy 和 cargo fmt 确保代码质量
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5, 6.3, 7.1, 7.2, 7.3, 7.4, 7.5, 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.2, 10.3, 10.4, 10.5_
