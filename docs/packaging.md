# 打包与发布

目标产物：Linux RPM / DEB，Windows NSIS 安装包。macOS 不发布。

## 本地打包（Linux）

```bash
cargo install cargo-deb cargo-generate-rpm
./scripts/package-linux.sh            # 构建并打包
./scripts/package-linux.sh --skip-build  # 复用已有 target/release/mdoxide
```

产物在 `dist/`：

| 文件 | 说明 |
|---|---|
| `mdoxide-editor_*.deb` | 安装到 `/usr/bin/mdoxide`，附带桌面文件与图标 |
| `mdoxide-editor-*.rpm` | 同上，RPM 元数据在 `Cargo.toml [package.metadata.generate-rpm]` |

安装验证：

```bash
sudo rpm -i dist/*.rpm      # 或 sudo dpkg -i dist/*.deb
mdoxide --version
```

## Windows

`installer/windows/mdoxide.nsi` 为 NSIS 脚本，安装到 `%ProgramFiles%\MDOxide`，创建桌面与开始菜单快捷方式，并注册卸载项。

```powershell
cargo build --release --locked
makensis /DVERSION=v0.1.0 installer/windows/mdoxide.nsi
```

## GitHub Actions

打 tag 触发 `.github/workflows/release.yml`：

```bash
git tag v0.1.0 && git push origin v0.1.0
```

流程：
1. Ubuntu 22.04 构建 release，生成 DEB + RPM
2. Windows 构建 release，调用 makensis 生成安装包
3. 所有产物作为 Release 附件上传（draft）

> 签名（Windows Authenticode / RPM GPG）尚未接入；M5 收尾时补。

## 元数据维护

- 应用 ID：`dev.mdoxide.app`
- 桌面文件：`assets/linux/dev.mdoxide.app.desktop`
- AppStream：`assets/linux/dev.mdoxide.app.metainfo.xml`
- 图标：`assets/icons/icon_*.png`（eframe 运行时）、`assets/icons/windows/app.ico`（Windows 资源）
- 版本号唯一来源：`Cargo.toml` 的 `version`
