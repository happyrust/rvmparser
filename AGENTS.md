# AGENTS.md

## Cursor Cloud specific instructions

This is a C++20 CLI tool (`rvmparser`) for parsing AVEVA PDMS RVM files and exporting to OBJ, GLTF/GLB, JSON, and `.rev` formats. There are no web services, databases, or Docker dependencies.

### Build (C++ / primary product)

```bash
cd make && CXXFLAGS=-fms-extensions CFLAGS=-fms-extensions make
```

- Produces `make/rvmparser` binary.
- Git submodules (`libs/libtess2`, `libs/rapidjson`) must be initialized before building. The `rvm-rs` submodule URL is broken and should be skipped; use `git submodule update --init libs/libtess2 libs/rapidjson` instead of `--recursive`.
- The CI uses `-fms-extensions` for the gcc build (see `.github/workflows/build.yml`). Without it, the build still succeeds but doesn't match CI exactly.
- Clang builds require `--gcc-install-dir=/usr/lib/gcc/x86_64-linux-gnu/13` on this VM because the default clang search path looks for GCC 14 headers.

### Testing

There is no automated test framework or `make test` target. The only test is a Windows batch script (`test/test-win32.bat`) that downloads sample RVM data from [pmuc](https://github.com/benvautrin/pmuc) and runs the parser. To test on Linux:

```bash
cd test
curl -L -o plm-sample_11072013.rvm https://github.com/benvautrin/pmuc/raw/master/data/plm-sample_11072013.rvm
curl -L -o plm-sample_11072013.att https://github.com/benvautrin/pmuc/raw/master/data/plm-sample_11072013.att
../make/rvmparser --output-json=plm-sample.json --output-obj=plm-sample --output-gltf=plm-sample.glb plm-sample_11072013.rvm plm-sample_11072013.att
```

### Lint

The Makefile uses `-Wall` by default. The codebase has some existing warnings (unused variables, reorder) that are known and present in CI. No separate lint tool (clang-tidy, cppcheck) is configured.

### Rust port (rvm-rs)

The `rvm-rs` submodule is non-functional — its remote URL (`./rvm-rs`) resolves to a non-existent GitHub repo. Ignore it unless the submodule is fixed upstream.
