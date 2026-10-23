# MDOxide

**简体中文** | [English](README.md)

用 Rust 编写的原生所见即所得 Markdown 编辑器，基于 `egui`/`eframe` —— 不使用 WebView、不打包 Electron、不依赖 JavaScript 运行时。

MDOxide 是对 [MarkText](https://github.com/marktext/marktext) 编辑体验的 Rust 重实现，代码基座来自 [Ferrite](https://github.com/OlaProeis/Ferrite)（MIT 协议）。

名字来源：**Oxide**（Rust 生态的氧化层意象）+ **MD**（Markdown）。

## 当前状态

早期开发阶段。M0（基座 Fork + 改名 + 代码审查）已完成，正在推进 M1（模块裁减与稳定性）与 M2（MarkText 外观重塑）。计划与审查报告见 [`docs/plans/`](docs/plans/)。

## 已具备的能力（继承自 Ferrite）

- 所见即所得 Markdown 编辑：按块切换渲染/编辑态，提交边界撤销
- CommonMark + GFM（`comrak`），代码高亮（`syntect`）
- 源码 / 渲染 / 分屏三种视图
- 原生 Mermaid 渲染（11 种图表，纯 Rust 实现）
- HTML 与 PDF 导出
- 文件树、工作区模式、正则查找替换
- 10 种界面语言

## 规划中

- 复刻 MarkText 视觉体系：无边框标题栏、居中标签页、三页签侧边栏、选区浮动格式条
- 与 Muya 对齐的打字机模式 / 专注模式
- 内置思源黑体，保证中文显示不依赖系统字体
- RPM / DEB / Windows NSIS 打包

## 构建

环境要求：Rust 1.92（由 `rust-toolchain.toml` 锁定），以及 Linux 图形开发包（Fedora：`gtk3-devel`、`libxkbcommon-devel`、`wayland-devel`）。

```bash
cargo run --release        # 运行
cargo test --release       # 1602 个测试，0 失败
cargo build --release      # 产物：target/release/mdoxide
```

## 许可证

MIT。MDOxide 派生自 Ferrite（版权所有 © Ola Proeis），并参考 MarkText 的交互设计；完整署名见 [`NOTICE`](NOTICE)。
