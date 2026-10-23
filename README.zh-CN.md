# MDOxide

**简体中文** | [English](README.md)

用 Rust 编写的原生所见即所得 Markdown 编辑器，基于 `egui`/`eframe` —— 不使用 WebView、不打包 Electron、不依赖 JavaScript 运行时。

MDOxide 是对 [MarkText](https://github.com/marktext/marktext) 编辑体验的 Rust 重实现，从一个 MIT 协议的 egui 编辑器基座起步，完整署名见 [`NOTICE`](NOTICE)。

名字来源：**Oxide**（Rust 生态的氧化层意象）+ **MD**（Markdown）。

## 当前状态

v0.1.0 范围内的功能已齐备。M0–M4 均已完成：基座 Fork 与代码审查、模块裁减、MarkText 外观重塑、交互对齐（含 fcitx5 IME 真机走查）、导出质量。M5（打包与发布收尾）进行中。计划与审查报告见 [`docs/plans/`](docs/plans/)。

## 已具备的能力

- 所见即所得 Markdown 编辑：按块切换渲染/编辑态，提交边界撤销
- 源码 / 渲染 / 分屏三种视图
- MarkText 风格界面：无边框标题栏、居中标签页、三页签侧边栏、选区浮动格式条、专注与打字机模式
- 内置思源黑体，中文渲染不依赖系统字体；fcitx5 IME 已在 Wayland 真机验证（预编辑、候选窗、组合中 Backspace、提交、Esc）
- CommonMark + GFM（`comrak`），代码高亮（`syntect`）
- 原生 Mermaid 渲染（11 种图表），HTML 导出内嵌 Mermaid SVG
- RaTeX 数学公式：编辑器内渲染，并嵌入 HTML/PDF 导出
- HTML 与 PDF 导出：内嵌中文字体，支持表格、代码块、链接与公式图片
- 文件树、工作区模式、正则查找替换
- 10 种界面语言

## 规划中

- 发布打包验证（RPM / DEB / Windows NSIS）
- PDF v1 已知限制：Mermaid 代码块、图片占位、bidi 重排

## 构建

环境要求：Rust 1.92（由 `rust-toolchain.toml` 锁定），以及 Linux 图形开发包（Fedora：`gtk3-devel`、`libxkbcommon-devel`、`wayland-devel`）。

```bash
cargo run --release        # 运行
cargo test --release       # 1601 个测试，0 失败
cargo build --release      # 产物：target/release/mdoxide
```

## 许可证

MIT。MDOxide 派生自 上游（版权所有 © Ola Proeis），并参考 MarkText 的交互设计；完整署名见 [`NOTICE`](NOTICE)。
