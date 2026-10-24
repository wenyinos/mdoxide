# MDOxide

**简体中文** | [English](README.md)

用 Rust 编写的原生所见即所得 Markdown 编辑器，基于 `egui`/`eframe` —— 不使用 WebView、不打包 Electron、不依赖 JavaScript 运行时。

MDOxide 是对 [MarkText](https://github.com/marktext/marktext) 编辑体验的 Rust 重实现，从一个 MIT 协议的 egui 编辑器基座起步，完整署名见 [`NOTICE`](NOTICE)。

名字来源：**Oxide**（Rust 生态的氧化层意象）+ **MD**（Markdown）。

## 当前状态

**v0.3.2 已发布。** 六个里程碑全部完成：基座 Fork 与代码审查、模块裁减、MarkText 外观重塑、交互对齐（含 fcitx5 IME 真机走查）、导出质量与打包。发布产物由 GitHub Actions 构建，覆盖 Linux x86_64/aarch64 与 Windows x64/ARM64。

## 已具备的能力

- 所见即所得 Markdown 编辑：按块切换渲染/编辑态，提交边界撤销
- 源码 / 渲染 / 分屏三种视图
- MarkText 风格界面：32px 无边框标题栏（汉堡应用菜单、视图切换、专注模式与设置快捷按钮、居中路径与未保存点）、28px 标签栏与主题色下划线、选区浮动格式条、打字机模式
- 专注模式淡化非当前编辑块；禅模式隐藏侧栏与状态栏，但保留顶部标签页
- 排版对齐 MarkText：正文 15px、1.6 倍行距；表格单元格自动换行并按内容增高
- 内置思源黑体，中文渲染不依赖系统字体；fcitx5 IME 已在 Wayland 真机验证（预编辑、候选窗、组合中 Backspace、提交、Esc）
- CommonMark + GFM（`comrak`），代码高亮（`syntect`）
- 原生 Mermaid 渲染（11 种图表），HTML 导出内嵌 Mermaid SVG
- RaTeX 数学公式：编辑器内渲染，并嵌入 HTML/PDF 导出
- HTML 与 PDF 导出：内嵌中文字体，支持表格、代码块、链接与公式图片
- 文件树、工作区模式、正则查找替换
- 10 种界面语言（996 个中英对齐翻译键）

## 已知限制

- PDF v1：Mermaid 输出为代码块、图片为占位符、未应用 bidi 重排
- 不支持 macOS

## 构建

环境要求：Rust 1.92（由 `rust-toolchain.toml` 锁定），以及 Linux 图形开发包（Fedora：`gtk3-devel`、`libxkbcommon-devel`、`wayland-devel`）。

```bash
cargo run --release        # 运行
cargo test --release       # 完整测试套件
cargo build --release      # 产物：target/release/mdoxide
```

## 许可证

MIT。MDOxide 派生自 上游（版权所有 © Ola Proeis），并参考 MarkText 的交互设计；完整署名见 [`NOTICE`](NOTICE)。
