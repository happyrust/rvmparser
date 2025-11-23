# RVM Parser - Rust Implementation

This repository contains a complete Rust port of the RVM parser, located in the `rvm-rs/` subdirectory.

## Overview

The Rust implementation provides:
- ✅ Complete RVM binary file parsing
- ✅ ATT attribute file parsing
- ✅ Export to OBJ, glTF/GLB, and JSON formats
- ✅ Scale-aware tessellation matching C++ implementation
- ✅ Memory-safe and type-safe implementation
- ✅ All tests passing with C++ parity

## Quick Start

```bash
cd rvm-rs

# Build the project
cargo build --release

# Run tests
cargo test

# Parse an RVM file
cargo run -- model.rvm

# Export to different formats
cargo run -- model.rvm --export-obj output.obj
cargo run -- model.rvm --export-gltf output.gltf
cargo run -- model.rvm --export-json output.json
```

## Documentation

- **Project Status**: [rvm-rs/PROJECT_STATUS.md](rvm-rs/PROJECT_STATUS.md)
- **Changelog**: [rvm-rs/CHANGELOG.md](rvm-rs/CHANGELOG.md)
- **Export Implementation**: [rvm-rs/EXPORT_IMPLEMENTATION.md](rvm-rs/EXPORT_IMPLEMENTATION.md)
- **Test Results**: [rvm-rs/TEST_RESULTS.md](rvm-rs/TEST_RESULTS.md)

## Specifications

Complete specification documents are available in `.kiro/specs/`:
- **rvm-rust-port**: Core parser implementation
- **rvm-export**: Export functionality (OBJ, glTF, JSON)
- **rvm-geometry-processing**: Geometry processing tools (planned)
- **rvm-hierarchy-tools**: Hierarchy manipulation tools (planned)

## Key Features

### Parser
- Binary RVM file format support
- All 11 geometry types: Pyramid, Box, Cylinder, Sphere, Circular Torus, Rectangular Torus, Elliptical Dish, Spherical Dish, Snout, Line, FacetGroup
- Transparency support for OBST and INSU types
- Attribute file parsing

### Export
- **OBJ/MTL**: Wavefront format with materials
- **glTF/GLB**: Industry-standard 3D format
- **JSON**: Structured data export
- Configurable options: centering, coordinate system conversion, tolerance

### Tessellation
- Scale-aware adaptive subdivision
- Sagitta-based segment calculation
- Matches C++ implementation quality

## Recent Updates (2024-11-23)

### Scale-Aware Tessellation Fix
- Implemented `get_scale()` function to extract scale from transformation matrices
- Added `sagitta_based_segment_count()` for adaptive tessellation
- Updated all geometry types to use scale-aware subdivision
- Verified matrix construction matches C++ implementation
- All 20 tests passing

## Performance

The Rust implementation provides:
- Comparable or better parsing speed than C++
- Memory safety through Rust's ownership system
- Type safety with strong typing
- Zero-cost abstractions

## Development

```bash
# Code quality checks
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt

# Build release version
cargo build --release

# Run specific tests
cargo test --lib
cargo test --test export_integration_test
```

## License

MIT License (consistent with the original C++ version)

## Contributing

The core functionality and export features are complete. Advanced features (geometry processing and hierarchy tools) have basic structure in place and can be developed as needed.

For more details, see the [Project Status](rvm-rs/PROJECT_STATUS.md) document.
