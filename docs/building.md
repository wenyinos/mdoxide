# 构建 MDOxide

目标平台：Linux（主）、Windows。macOS 不在支持范围内。

## 环境要求

- **Rust 1.92+**（版本由 `rust-toolchain.toml` 锁定）
- **Git**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version && cargo --version
```

### Linux（Fedora）

```bash
sudo dnf install gtk3-devel libxkbcommon-devel wayland-devel \
                 libxcb-devel fontconfig-devel
```

Debian/Ubuntu 对应：

```bash
sudo apt install build-essential pkg-config libgtk-3-dev \
                 libxkbcommon-dev libwayland-dev libxcb1-dev libfontconfig1-dev
```

### Windows

- Visual Studio Build Tools 2019+（C++ 工作负载）
- WiX Toolset 3.11+（打包 MSI/NSIS 时需要）

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

## 常用命令

```bash
cargo run                     # 开发运行（debug）
cargo run --release           # 开发运行（优化）
cargo build --release         # 产物：target/release/mdoxide（Linux）/ mdoxide.exe（Windows）
cargo test --release          # 单元测试（当前 1583 通过）
cargo check                   # 快速类型检查
cargo clippy --release        # 静态检查（基线 261 条告警，尚未清零）
cargo fmt                     # 格式化
```

## 构建产物与体积

release 构建启用了 LTO、单 codegen unit、符号剥离与 panic=abort，Linux 二进制约 15–20MB。

## 打包（M5 阶段落地）

| 平台 | 格式 | 工具 |
|------|------|------|
| Linux | RPM | `cargo-generate-rpm` |
| Linux | DEB | `cargo-deb` |
| Windows | NSIS | `cargo-packager` 或 NSIS 脚本 |

> 旧 上游 时代的 MSI（`wix/`）与 PortableApps（`portable/`）配置已移除。

## 常见问题

**Q: 启动报 GTK / 显示相关错误？**
确认已安装 GTK3 开发包与 Wayland/X11 开发包；Wayland 下如需 X11 兼容可设置 `WAYLAND_DISPLAY=` 强制走 X11。

**Q: 中文显示为方块？**
M2 之前仍依赖系统字体（Noto Sans CJK SC 等）；M2 会内置思源黑体，届时不再依赖系统。
