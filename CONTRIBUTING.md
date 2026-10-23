# 参与 MDOxide 开发

## 环境

Rust 1.92（`rust-toolchain.toml` 锁定），Linux 需 GTK3 / xkbcommon / Wayland 开发包。详见 [`docs/building.md`](docs/building.md)。

## 提交前检查

```bash
cargo fmt
cargo clippy --release      # 允许现有告警，但不得新增
cargo test --release        # 必须全绿（当前 1583 通过 / 3 忽略）
```

## 原则

- **一次提交一件事**，提交信息用 `feat:` / `fix:` / `refactor:` / `docs:` / `chore:` 前缀
- **不引入 WebView / Electron / JS 运行时**——这是项目的硬约束
- **内核改动必须带测试**：`src/editor/`、`src/markdown/`、`src/state.rs` 的编辑/撤销/持久化路径
- **不动未涉及的文件**，不顺手重构，不新增「为未来预留」的抽象
- **UI 代码放独立模块**，不要继续往 `state.rs`（7.9k 行）、`markdown/editor.rs`（7.1k 行）、`markdown/widgets.rs`（6.4k 行）里堆
- 新增依赖前先说明理由；默认拒绝为小功能引入大依赖

## 上游关系

本项目的代码基座来自一个 MIT 协议的 egui 编辑器（署名见 [`NOTICE`](NOTICE)）。仓库保留了名为 `upstream` 的 remote，仅用于挑选底层修复：

```bash
git fetch upstream
git cherry-pick <commit>      # 不要直接 merge 上游 UI 改动
```

品牌、UI 与打包配置属于本项目自有改动，不与上游同步。

## 许可证

提交即表示同意以 MIT 协议授权你的贡献。
