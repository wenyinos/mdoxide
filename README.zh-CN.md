# MDOxide

**简体中文** | [English](README.md)

用 Rust 编写的原生所见即所得 Markdown 编辑器，基于 `egui`/`eframe` —— 不使用 WebView、不打包 Electron、不依赖 JavaScript 运行时。

MDOxide 是对 [MarkText](https://github.com/marktext/marktext) 编辑体验的 Rust 重实现，从一个 MIT 协议的 egui 编辑器基座起步，完整署名见 [`NOTICE`](NOTICE)。

名字来源：**Oxide**（Rust 生态的氧化层意象）+ **MD**（Markdown）。

## 当前状态

**v0.8.5 已发布。** 从 MarkText `0.24.0-dev` 移植 12 项特性（行为级重写）：脚注渲染（首次引用顺序编号）、标题章节折叠（含折叠全部/展开全部）、Slash 快速插入菜单（标题/列表/引用/代码/表格/公式/Mermaid/分隔线）、Emoji 短代码补全、6×8 表格棋盘、图片拖拽缩放（写回 `<img width>`）、GitHub 风格标题锚点、取消链接、复制为 HTML、图片悬停工具，以及设置搜索纳入键盘快捷键。此前版本覆盖 Ferrite v0.3.1 可移植项、UI 缩放持久化与告警清零。发布产物由 GitHub Actions 构建，覆盖 Linux x86_64/aarch64 与 Windows x64/ARM64。

## 已具备的能力

- 所见即所得 Markdown 编辑：按块切换渲染/编辑态，提交边界撤销
- 源码 / 渲染 / 分屏三种视图
- MarkText 风格界面：32px 无边框标题栏（汉堡应用菜单、视图切换、专注模式与设置快捷按钮、居中路径与未保存点）、28px 标签栏与主题色下划线、选区浮动格式条、打字机模式
- 源码与预览模式文本右键菜单：复制 / 剪切 / 粘贴 / 全选；预览可“复制全部（纯文本）”或“复制为 Markdown”
- 专注模式淡化非当前编辑块；禅模式隐藏侧栏与状态栏，但保留顶部标签页
- 排版对齐 MarkText：正文 15px、1.6 倍行距；表格单元格自动换行并按内容增高
- 内置思源黑体，中文渲染不依赖系统字体；fcitx5 IME 已在 Wayland 真机验证（预编辑、候选窗、组合中 Backspace、提交、Esc）
- CommonMark + GFM（`comrak`），代码高亮（`syntect`）
- 原生 Mermaid 渲染（11 种图表），HTML 导出内嵌 Mermaid SVG
- RaTeX 数学公式：编辑器内渲染，并嵌入 HTML/PDF 导出
- HTML 与 PDF 导出：内嵌中文字体，支持表格、代码块、链接与公式图片
- 文件树、工作区模式、正则查找替换
- 修订号驱动编辑器与缓存失效；外部修改只有一个入口
- 正文、路径、编码与撤销历史统一由 `Document` 持有（`Tab` 委托访问）
- 富 Markdown 块：`<details>/<summary>`、对齐 `<div>`、`kbd/sup/sub`、Mermaid 点击弹窗与完整 gitGraph 属性
- 渲染视图方向键块导航、预览锁定与 Frontmatter 属性卡片
- 编辑器/渲染/终端独立字体，支持自定义系统字体与 Nerd Font 字形回退
- 设置逐行搜索、运行时统计（字体/Mermaid 缓存/终端）、原生标题栏与 Linux 主选区粘贴
- Raw 编辑按操作上报，不再每键整篇转换与 diff
- 配置、会话与恢复文件使用耐久原子写；配置保留上一版备份
- 崩溃恢复由后台单写线程序列化；工作区索引带 400ms 合并且单 worker
- 代码执行 2 MB 输出上限并终止整个进程树；HTML 活动内容默认关闭；终端布局默认不自动加载
- 10 种界面语言（1006 个中英对齐翻译键）

## 已知限制

- PDF v1：Mermaid 输出为代码块、图片为占位符、未应用 bidi 重排
- 预览选中以块为单位，暂不支持跨段拖选（可用“复制全部”复制整篇）
- 不支持 macOS

## 构建

环境要求：stable Rust（由 `rust-toolchain.toml` 指定），以及 Linux 图形开发包（Fedora：`gtk3-devel`、`libxkbcommon-devel`、`wayland-devel`）。

```bash
cargo run --release        # 运行
cargo test --release       # 完整测试套件
cargo build --release      # 产物：target/release/mdoxide
```

## 许可证

MIT。MDOxide 派生自 上游（版权所有 © Ola Proeis），并参考 MarkText 的交互设计；完整署名见 [`NOTICE`](NOTICE)。
