# 更新日志

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循语义化版本。

## [0.1.2] - 2026-10-24

### 新增

- 界面汉化补全：新增 90 余个中文翻译键，覆盖命令面板、设置面板、导出对话框、错误提示、欢迎页、frontmatter 面板等模块
- 启动语言默认跟随系统区域设置；用户在设置中显式选择语言后才固定该选择
- 界面与编辑器默认字号由 14px 提升到 15px，中文界面优先使用思源黑体，改善小字号下的可读性

### 修复

- Windows 安装包构建失败：release workflow 现在安装 NSIS 并从 tag 解析版本号
- 界面残留英文文案：命令面板分类、frontmatter 面板、欢迎页、语言服务器设置、PDF 预览错误等

## [0.1.0] - 2026-10-24

首个开发版本，基于上游 v0.3.0（commit `3ba085c`）。

### 新增

- 保存原子化：`files::atomic::write_atomic`（临时文件 → fsync → 原子替换，保留权限），Save / Save As 全部接入
- 数学公式导出：HTML 与 PDF 均通过 RaTeX 渲染为内联 PNG；HTML 暗色主题自动反色
- PDF 导出嵌入思源黑体，中文不再缺字
- 手动导出走查测试（`cargo test --bin mdoxide walkthrough -- --ignored`）

### 变更

- 项目更名为 MDOxide（原上游项目），crate、应用 ID、配置目录、环境变量、桌面文件同步更新
- Markdown AST 缓存改为 `Arc<MarkdownDocument>`，缓存命中不再深拷贝
- CI 改为 push 到 master、PR、手动触发三种方式

### 移除

- Git 集成（`src/vcs`、`git2` 依赖、文件树状态徽章、状态栏分支指示、自动刷新）
- Nix 支持（`flake.nix`、`flake.lock`、Nix 工作流与相关文档）
- 上游遗留文档与产物：`.planning/`、`docs/ai-workflow/`、`portable/`、`wix/`、`.signpath/`、上游截图与品牌图、84MB 测试 CSV
- `release.yml`（上游专属签名/打包配置），M5 阶段重建

### 修复

- 会话恢复：无身份恢复文件不得覆盖有路径的标签（数据丢失缺陷）
- 上游 42 个测试编译错误（`TextBuffer` 导入、Mermaid 测试路径、图标常量、`expand_rect` 导入、`Task::to_markdown` 缺失）
- PDF 导出：表格边框误填充为深色（绘制状态泄漏）、中文长段落不换行且溢出右边距、行首标点、行内公式撑破行高、链接颜色被正文色覆盖、任务列表标记缺字
