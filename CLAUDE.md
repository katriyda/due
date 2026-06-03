# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

due — Windows 桌面提醒应用，Rust + Slint GUI，后台托盘运行，TOML 格式持久化。

## 构建与测试

```powershell
cargo check          # 编译检查
cargo test           # 运行全部测试
cargo test <name>    # 运行单个测试
cargo test reminder::  # 运行指定模块的测试
cargo run            # 运行应用
```

## 约束

- **禁止抑制错误/警告**：不允许 `#![allow(dead_code)]`、`#[allow(warnings)]` 等。未使用的代码要么被使用，要么被移除。
- **禁止滥用 `unwrap()`**：可能失败的地方使用 `Result` 和 `?`。测试中可以 `unwrap()`。
- **TDD 垂直切片**：一次一个测试 → 一次一个实现。不要水平切片（先写全部测试再写全部实现）。
- **测试只验证公共行为**：不测实现细节，不 mock 内部模块。测试应能承受重构。

## 架构

```
src/
├── main.rs          # 入口：加载数据，连接 Slint UI 回调，运行事件循环
├── reminder.rs      # Reminder 模型 + TOML 存储（save_reminders / load_reminders / data_dir）
├── config.rs        # AppConfig 模型 + TOML 配置
├── ui.rs            # UI 数据转换（ReminderItem）+ 编辑/添加/删除/保存逻辑
├── time.rs          # 时间解析（本地/相对/中文）+ 重复触发判断
├── popup.rs         # 弹窗操作（Dismiss / Snooze / Complete）
├── notification.rs  # Windows 系统通知（notify-rust）
└── tray.rs          # 系统托盘（tray-icon + muda 菜单）
ui/
└── main.slint       # Slint GUI 定义（主窗口布局、编辑面板、回调声明）
build.rs             # slint_build::compile("ui/main.slint")
```

### 数据流

Reminder 模型 (`reminder.rs`) → UI 转换 (`ui::to_reminder_items`) → Slint 渲染 (`main.slint`)
用户操作 → Slint 回调 → `ui.rs` 逻辑 → `reminder.rs` 持久化 → 刷新 UI

### 存储

- 数据目录：`%APPDATA%\due\`（通过 `dirs::data_dir()` 获取）
- 提醒文件：`reminders.toml`（TOML 格式，`[[reminders]]` 数组）
- 配置文件：`config.toml`（TOML 格式，首次运行自动生成默认配置）

### Slint UI 注意事项

- Slint 标识符用连字符（`repeat-type-index`），Rust 端自动转为下划线（`repeat_type_index`）
- `if` 块内声明的 ID 无法从外部访问，需通过 `in-out property` 中转
- `build.rs` 编译 `ui/main.slint` 生成 Rust 绑定，`slint::include_modules!()` 引入

### 术语

使用 `CONTEXT.md` 中定义的领域术语（提醒/通知/延后等），不要用同义词。

## 领域文档

- **`CONTEXT.md`** — 术语表和领域概念定义
- **`docs/adr/`** — 架构决策记录（如 ADR-0001: 选择 Slint）
- **`docs/prd-reminder-app.md`** — 产品需求文档

## Agent Skills

### Issue tracker

GitHub Issues，通过 `gh` CLI 操作。详见 `docs/agents/issue-tracker.md`。

### Triage labels

默认标签：needs-triage / needs-info / ready-for-agent / ready-for-human / wontfix。详见 `docs/agents/triage-labels.md`。
