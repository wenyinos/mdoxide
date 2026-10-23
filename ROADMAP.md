# MDOxide 路线图

以「用 Rust 复刻 MarkText 的编辑体验」为唯一目标，分六个里程碑推进。详细拆解见 [`docs/plans/实施计划.md`](docs/plans/实施计划.md)。

## 已完成

- **M0 基座**：Fork 上游项目（MIT），全仓改名，修复上游 42 个测试编译错误，修复会话恢复的数据丢失缺陷
- **M1 稳定与裁减**：保存原子化（临时文件 + fsync + rename）、AST 缓存 Arc 化、移除 Git 集成、移除 Nix/Release 工作流与 84MB 测试数据

## 进行中

- **M2 外观重塑**（已完成约 85%）：
  - ✅ MarkText 亮/暗配色、`#409eff` 强调色、720px 居中阅读栏
  - ✅ 内置思源黑体（子集，Regular 3.5MB + Bold 3.6MB）
  - ✅ 标签栏按行居中，保留拖拽排序
  - ✅ 侧边栏三页签（大纲 / 文件 / 搜索），其余面板收进溢出菜单
  - ✅ 打字机模式（原始编辑器）
  - ⬜ 选区气泡格式条、专注模式、渲染模式下的打字机居中

## 计划中

- **M3 交互对齐**：光标所在块源码标记淡显、选区浮动格式条、打字机/专注模式、MarkText 键位表、fcitx5 IME 验证
- **M4 导出与扩展**：HTML/PDF 导出质量走查（hayro → typst 备选）、front matter、emoji；数学公式与 PlantUML 首版降级
- **M5 发布**：偏好设置面板、拼写检查、大文件性能、RPM / DEB / Windows NSIS 打包、v0.1.0

## 明确不做

- macOS 平台支持
- Nix / NixOS 打包
- WebView、Electron、JavaScript 运行时
- PlantUML 渲染（首版）
- LSP 集成
