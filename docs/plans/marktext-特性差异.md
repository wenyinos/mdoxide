# MarkText 0.24.0-dev 特性差异与移植计划

> 上游基线：`/home/zemi/MyWork/marktext`，`0.24.0-dev`，`master` `95a14e5c`（2026-11）
> 对照基线：MDOxide `v0.8.1`（2026-11-25）
> 方法：读 Muya 块/控件清单、`git log` 近期 feat 提交、desktop renderer 设置项，逐项与 MDOxide 现有能力比对；每项移植按本分支 AST/渲染模型重写，不照搬 JS 实现

## 一、MDOxide 已覆盖

CommonMark/GFM 块与内联、表格（自动换行/行高）、任务列表、代码块（语法高亮）、引用块、GitHub Alerts（Callout）、数学公式（RaTeX）、Mermaid 11 类图表、Frontmatter 卡片、`<details>/<summary>`、`<div align>`、`<kbd>/<sup>/<sub>` 标签剥离、视频嵌入、Wikilink、HTML/PDF 导出、源码/渲染/分屏三视图、专注模式、打字机模式、块级键盘导航、气泡格式工具栏、搜索替换（Unicode 大小写折叠）、大纲、文件树、表情短代码显示、图片粘贴、主题（含 Duskfox Blurred）、i18n。

## 二、缺口清单

### A. 渲染层缺口（解析已有、渲染缺失）

| 编号 | 特性 | 上游实现 | 本分支现状 | 优先级 | 实施要点 |
|---|---|---|---|---|---|
| R-1 | 脚注渲染 | `block/extra/footnote`：正文引用显示编号，文末定义带回跳 | comrak 已解析（`FootnoteReference`/`FootnoteDefinition`），渲染落 fallback | **已实施** | 预扫描引用顺序建 `name→number` 映射；内联小字引用；定义块渲染 `[n]` + 子块内容；3 个回归测试 |
| R-2 | 上标 `^text^` | `inline` 层 `sup` 投影，偏好默认关 | 渲染分支已加（0.72× 小字号）；`MarkdownOptions.superscript` 默认 false，与 MarkText 一致 | **部分** | 待加「启用上标」设置项并透传解析选项 |
| R-3 | 描述列表 | **MarkText 不支持** | comrak `description_lists` 默认关；渲染分支已就绪 | 可选扩展 | 非 MarkText 特性，保留为关闭态扩展 |
| R-4 | 引用式链接 | `referenceDefinitions` / `referenceLabel` | comrak 默认解析为 Link，节点已覆盖但无 label 保留 | P2 | 当前可用；label 保留需求低，暂缓 |

### B. 编辑器块交互

| 编号 | 特性 | 上游实现 | 本分支现状 | 优先级 | 实施要点 |
|---|---|---|---|---|---|
| E-1 | 标题折叠 | `headingFoldToggle` + `folding.ts`：按级别折叠章节、嵌套、状态保存 | **已实施**：标题左侧 ▾/▸ 折叠按钮，按级别隐藏后续块 | 已实施 | 折叠态存 `RenderedEditSession.folded_headings`（瞬态，不写回正文）；View 菜单提供折叠全部/展开全部；culling 按折叠签名失效并零高度隐藏；块导航跳过隐藏块；行号键在编辑后可能漂移（MarkText 用 path，后续可改进） |
| E-2 | 表格棋盘插入 | `ui/tableChessboard`：悬浮 6×8 棋盘选行列 | **已实施**：Format 菜单 → 插入表格 → 6×8 棋盘 | P1 | `MarkdownFormatCommand::Table{rows,cols}` + `apply_table_snippet_format` + `table_picker_grid`；4 个测试 |
| E-3 | 代码块语言选择器 | `ui/codeBlockLanguageSelector`：点击语言弹模糊搜索 | **已覆盖**：代码块标题栏 ComboBox 可选语言并回写 fence | — | 交互形态不同（下拉 vs 搜索），功能等价 |
| E-4 | 图片缩放 | `ui/imageResizeBar`：拖拽角柄改宽度 | 图片固定渲染，无尺寸控制 | P1 | 拖拽生成 `width` 属性（HTML img 或 `{width=}`），需 AST/源文同步 |
| E-5 | 快速插入菜单 | `paragraphQuickInsertMenu`：段落输入 `/` 或 `、` 弹块类型候选 | **已实施**：段落编辑态输入 `/`/`、` 触发候选浮层 | 已实施 | `quick_insert_query` 检测；候选 12 项（标题/列表/任务/引用/代码/表格/公式/Mermaid/分隔线）；Enter/点击应用，Esc 关闭并记忆；复用块提交路径 |
| E-6 | Emoji 选择器 | `ui/emojiSelector`：`:` 触发补全 | **已实施**：段落编辑态 `:prefix` 弹 emoji 列表 | 已实施 | 复用 slash 菜单弹层；`emoji_query` 过滤、最多 24 项、插入 `:name: `；完成短代码不再触发 |
| E-7 | 表格拖拽条/列菜单 | `tableDragBar` / `tableRowColumMenu` | 无 | P2 | 列宽拖拽、插入/删除行列、对齐切换 |
| E-8 | 链接工具 | `ui/linkTools`：打开/编辑/复制/取消 | **已覆盖**：点击编辑弹窗含打开/复制/取消链接；Ctrl+点击浏览器打开 | — | 取消链接保留文本、剥离行内语法 |
| E-9 | 脚注工具 | `ui/footnoteTool` | 无 | P2 | 插入脚注引用并跳转定义 |
| E-10 | 段落缩进 | `paragraphIndentation` | 有 Tab 缩进与段落缩进设置 | 已覆盖 | — |
| E-11 | 列表操作 | `listOperations`：升降级/合并/拆分 | Tab/Shift+Tab 缩进已有，合并拆分未覆盖 | P2 | 回车/退格边界行为对齐 |
| E-12 | 滚动翻页 | `scrollPage` | 普通滚动 | P3 | PageUp/Down 语义对齐 |
| E-13 | 标题复制链接 | `headingCopyLink` | 无 | P3 | 标题 hover 复制锚点 |

### C. 桌面层

| 编号 | 特性 | 上游实现 | 本分支现状 | 优先级 | 实施要点 |
|---|---|---|---|---|---|
| D-1 | CJK 强调兼容偏好 | #6021：`**中文**` 与 CJK 标点相邻时按 CJK 规则识别 | comrak 解析，未暴露偏好 | P2 | 需解析器层定制，评估 comrak 可配置项 |
| D-2 | 标签页拖拽排序 | #5940 | **已覆盖**：`click_and_drag` + `DragAndDrop` payload + 落点高亮，`swap_tabs` 重排 | — | 交互已存在，无需移植 |
| D-3 | 侧栏拖拽移动文件 | #5945 | 无 | P2 | 文件树拖拽 + 移动确认 |
| D-4 | 主题选择器带预览 | #5934 | 下拉选择 | P3 | 预览卡片 |
| D-5 | 快捷键纳入设置搜索 | #5930 | **已覆盖**：统一搜索框展示匹配命令与当前绑定 | — | 只读展示，编辑仍在 Keyboard 分区 |
| D-6 | 快捷键冲突提示 | #5933 | 无 | P3 | 设置页标注冲突 |
| D-7 | 只读/阅读模式 | #5801/#5793 | 无 | P2 | 每标签只读标记，禁编辑保留导航 |
| D-8 | 预览标签页 | #5820 | 无 | P3 | — |
| D-9 | 多文件夹根 | #5835/#5836 | 单工作区 | P2 | 工作区模型扩展，影响面大 |
| D-10 | 最近文件进快速打开 | #5864 | 有最近文件列表 | 部分 | 快速打开面板合并最近项 |
| D-11 | 大纲折叠/展开全部 | #5867 | **已覆盖**：标题折叠/展开（双击或图标） | — | 无“全部展开”一键项，价值低 |
| D-12 | Pandoc fancy_lists | #5754 | 不支持 `I.`/`a.`/`(1)` | P2 | 解析器前置转换 |
| D-13 | 有序字体回退 | #5944 | 字体族有序回退已有 | 部分 | 偏好 UI |
| D-14 | Mermaid 明暗主题偏好 | #5862 | 跟随应用主题 | 已覆盖 | — |
| D-15 | 只读 CLI | #5821 | 无 | P3 | 参数解析 |
| D-16 | Copy as 块菜单 | #5958 | **已覆盖**：预览右键含复制全文/复制为 Markdown/复制为 HTML | — | HTML 复用剪贴板导出管线 |
| D-17 | 多窗口恢复 | #5922 | 明确不做 | — | — |

## 三、实施批次

- **第一批（v0.8.2 候选，本次实施）**：R-1 脚注渲染、R-2 上标渲染、R-3 描述列表渲染
- **第二批**：E-2 表格棋盘插入、E-3 代码块语言选择器
- **第三批（E-1 已完成）**：E-1 标题折叠；E-4 图片缩放待做
- **第四批（E-5/E-6 已完成）**：E-5 快速插入菜单、E-6 Emoji 选择器；E-8 链接工具待做
- **桌面层**：D-2、D-3、D-7、D-12、D-1 按需排期

## 四、不做

多窗口、拼写检查、LSP、macOS、AppImage、WebView/Electron 相关（沿用既有边界）。
