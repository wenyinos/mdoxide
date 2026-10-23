# MDOxide 品牌规范

> 状态：占位阶段。当前图标仍是 上游遗留的绿色 "F" 占位图标，M5 阶段替换为 MDOxide 自有图标。

## 品牌定位

**MDOxide** —— 用 Rust 编写的原生 Markdown 编辑器，无 WebView、无 Electron。
名字 = **MD**（Markdown）+ **Oxide**（Rust 生态意象）。

## 品牌色

| 用途 | 色值 | 说明 |
|------|------|------|
| 主色 | `#B7410E` | 氧化铁红（rust 的字面意象），用于强调、选中态 |
| 深色背景 | `#1E1E1E` | 暗色主题基底 |
| 浅色背景 | `#FAFAFA` | 亮色主题基底 |
| 正文（亮） | `#1F1F1F` | |
| 正文（暗） | `#E6E6E6` | |

> 主题色值最终以 `src/theme/` 中的实现为准，M2 主题移植时统一校准。

## 资源位置

```
assets/icons/
├── icon_16.png … icon_512.png      # 应用图标（eframe 运行时读取 icon_256.png）
├── windows/{app.ico, app.rc}       # Windows 可执行文件图标（build.rs 嵌入）
└── linux/<size>/dev.mdoxide.app.png  # Linux 图标，按 XDG 目录安装
assets/linux/dev.mdoxide.app.desktop # 桌面入口
assets/linux/dev.mdoxide.app.metainfo.xml
```

## 生成图标

```bash
cd assets/icons
python3 generate_icons.py     # 需要 Pillow
```

## Linux 安装

```bash
cp assets/linux/dev.mdoxide.app.desktop ~/.local/share/applications/
for size in 16 32 48 64 128 256 512; do
  mkdir -p ~/.local/share/icons/hicolor/${size}x${size}/apps
  cp assets/icons/linux/${size}x${size}/dev.mdoxide.app.png \
     ~/.local/share/icons/hicolor/${size}x${size}/apps/dev.mdoxide.app.png
done
gtk-update-icon-cache ~/.local/share/icons/hicolor/
```

## 使用规范

- 图标四周保留 ≥10% 留白，不拉伸、不改色
- 不在复杂纹理背景上使用
- 不将图标用于非 MDOxide 项目
