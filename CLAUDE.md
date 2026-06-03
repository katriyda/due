# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

due — Windows 桌面提醒应用，Rust + Slint GUI，后台托盘运行，TOML 格式持久化。

## 构建与测试

需要 MSVC 构建工具（Visual Studio Build Tools），编译/测试需在 MSVC 环境下运行：

```powershell
# 编译检查
cmd /c "`"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvarsall.bat`" x64 && cargo check"

# 运行全部测试
cmd /c "`"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvarsall.bat`" x64 && cargo test"

# 运行单个测试
cmd /c "`"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvarsall.bat`" x64 && cargo test <test_name>"

# 运行指定模块的测试
cmd /c "`"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvarsall.bat`" x64 && cargo test reminder::"
```

## 约束

- **禁止抑制错误/警告**：不允许 `#![allow(dead_code)]`、`#[allow(warnings)]` 等。未使用的代码要么被使用，要么被移除。
- **禁止滥用 `unwrap()`**：可能失败的地方使用 `Result` 和 `?`。测试中可以 `unwrap()`。
- **TDD 垂直切片**：一次一个测试 → 一次一个实现。不要水平切片（先写全部测试再写全部实现）。
- **测试只验证公共行为**：不测实现细节，不 mock 内部模块。测试应能承受重构。

## 架构

```
src/
├── main.rs          # 入口：加载数据目录，显示提醒列表
├── config.rs        # AppConfig 模型 + TOML 配置（load_config / save_config / ensure_config_file）
└── reminder.rs      # Reminder 模型 + TOML 存储（save_reminders / load_reminders / data_dir）
```

### 存储

- 数据目录：`%APPDATA%\due\`（通过 `dirs::data_dir()` 获取）
- 提醒文件：`reminders.toml`（TOML 格式，`[[reminders]]` 数组）
- 配置文件：`config.toml`（TOML 格式，首次运行自动生成默认配置）
- 配置格式：TOML

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
