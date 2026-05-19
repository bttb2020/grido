# Grido

GPU 加速的原生表格式数据编辑器，面向开发者。Rust + Makepad。

## 快速开始

```bash
cargo run -p grido-app                    # 运行应用
cargo run -p grido-app -- path/to/file.csv  # 打开文件
cargo test --workspace                    # 运行所有测试
cargo clippy --workspace                  # lint
```

## 目录结构

```
crates/
  grido-app/       主应用入口，Makepad UI 组装
  grido-core/      数据模型、编辑引擎（Document, CellValue, 撤销/重做）
  grido-grid/      网格渲染组件（虚拟滚动、单元格 shader、选区）
  grido-io/        文件格式读写（CSV, JSON, Parquet）
  grido-formula/   公式引擎（V1 阶段启用）
resources/         图标、字体等资源
tests/fixtures/    测试用数据文件
```

## 详细需求

完整 PRD 在 `~/minji/research/ai-product/gpui-spreadsheet-editor/PRD.md`。
