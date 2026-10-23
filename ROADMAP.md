# MDOxide 路线图

以「用 Rust 复刻 MarkText 的编辑体验」为唯一目标，分六个里程碑推进。详细拆解见 [`docs/plans/实施计划.md`](docs/plans/实施计划.md)。

## 已完成

- **M0 基座**：Fork 上游项目（MIT），全仓改名，修复上游 42 个测试编译错误，修复会话恢复的数据丢失缺陷
- **M1 稳定与裁减**：保存原子化（临时文件 + fsync + rename）、AST 缓存 Arc 化、移除 Git 集成、移除 Nix/Release 工作流与 84MB 测试数据
- **M3 交互对齐**：
  - ✅ 打字机/专注模式、选区浮动格式条、标题/引用源码标记淡显
  - ✅ MarkText 键位表（Ctrl+B/I/K、Ctrl+1–6、Ctrl+Shift+B/N、Ctrl+Q）
  - ✅ IME 组合输入：预编辑/提交/取消状态机、预编辑绘制、候选窗定位（6 项自动化测试覆盖，含 `ui()` 端到端路径）
  - ⏳ 真机 fcitx5 走查按 [`docs/plans/M3-走查清单.md`](docs/plans/M3-走查清单.md) 手工确认

## 进行中

- **M2 外观重塑**（已完成约 85%）：
  - ✅ MarkText 亮/暗配色、`#409eff` 强调色、720px 居中阅读栏
  - ✅ 内置思源黑体（子集，Regular 3.5MB + Bold 3.6MB）
  - ✅ 标签栏按行居中，保留拖拽排序
  - ✅ 侧边栏三页签（大纲 / 文件 / 搜索），其余面板收进溢出菜单
  - ✅ 打字机模式（原始 + 渲染）、专注模式、选区浮动格式条

## 计划中

- **M4 导出与扩展**：emoji 短代码渲染已完成；数学公式选型定为 **RaTeX**（纯 Rust、KaTeX 兼容、SVG/PNG 输出）；待办：HTML/PDF 导出质量走查（hayro → typst 备选）
- **M5 发布**：RPM / DEB / Windows NSIS 打包脚本与 release 工作流已就位（`scripts/package-linux.sh`、`installer/windows/mdoxide.nsi`）；偏好设置面板、大文件性能仍待完成

## 明确不做

- macOS 平台支持
- Nix / NixOS 打包
- WebView、Electron、JavaScript 运行时
- PlantUML 渲染（首版）
- 拼写检查（产品决策取消，不进入任何里程碑）
- LSP 集成
