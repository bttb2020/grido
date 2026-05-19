# Grido — Agent 协作指南

## 项目简介

Grido 是一个 GPU 加速的原生表格式数据编辑器，面向开发者，用于高效查看和编辑结构化数据文件（CSV、JSON、Parquet 等）。

**一句话**：Zed 的速度 × Excel 的交互 × 开发者的审美。

## 技术栈

- **语言**: Rust（全栈，无 FFI）
- **UI 框架**: Makepad（GPU shader-first 渲染，MIT 开源）
- **GPU 后端**: Metal (macOS) / Vulkan (Linux) / D3D11 (Windows) / WebGL (WASM)
- **构建**: Cargo workspace

## 构建和运行

```bash
# 运行
cargo run -p grido-app

# 打开文件
cargo run -p grido-app -- test.csv

# 测试
cargo test --workspace

# Lint（必须零 warning）
cargo clippy --workspace -- -W clippy::all

# 格式化
cargo fmt --all
```

## 目录结构

```
crates/
  grido-app/       主应用入口，Makepad UI 组装
  grido-core/      数据模型、编辑引擎
  grido-grid/      网格渲染组件
  grido-io/        文件格式读写
  grido-formula/   公式引擎（V1 阶段）
resources/         图标、字体
tests/fixtures/    测试数据文件
```

## Crate 职责

| Crate | 职责 | 核心类型 |
|-------|------|---------|
| `grido-app` | 应用入口、窗口管理、全局状态 | `App` |
| `grido-core` | 文档模型、单元格值、列定义、编辑操作、撤销/重做 | `Document`, `CellValue`, `EditCommand` |
| `grido-grid` | 网格 widget、虚拟滚动、单元格渲染 shader、选区 | `GridView`, `Viewport` |
| `grido-io` | CSV/TSV/JSON/Parquet 读写、格式检测、编码检测 | `FileFormat`, `detect_format()` |
| `grido-formula` | 公式解析、求值、依赖图、内置函数 | `Expr`, `FormulaEngine` |

## 代码规范

- `cargo fmt` — 所有代码必须格式化
- `cargo clippy` — 零 warning
- 每个公开 API 必须有 doc comment
- 错误处理用 `thiserror` 定义错误类型，禁止 `unwrap()`（测试除外）
- 命名遵循 Rust 标准：`snake_case` 函数/变量，`CamelCase` 类型

## 提交规范

- 前缀：`feat` / `fix` / `refactor` / `test` / `perf` / `docs` / `chore`
- 格式：`feat(grid): implement virtual scrolling`
- 每个功能点独立提交

## 性能目标

| 指标 | 目标 |
|------|------|
| 冷启动到首屏 | < 500ms |
| 打开 100MB CSV | < 1s |
| 滚动帧率 | ≥ 60fps |
| 单元格编辑延迟 | < 16ms |
| 内存（1M 行 × 20 列） | < 500MB |

## Makepad 开发要点

- UI 用 `script_mod!` 宏内联 DSL 定义布局和样式
- 事件处理实现 `MatchEvent` trait
- 自定义渲染用 `DrawText`, `DrawQuad` 等，可内联 shader
- 参考 Makepad 官方 examples: `counter`, `hello_world`, `scratchpad`

## 详细需求

完整 PRD 见 `~/minji/research/ai-product/gpui-spreadsheet-editor/PRD.md`
