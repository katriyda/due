# due

Windows 桌面提醒应用，Rust + Slint GUI，后台托盘运行。

## 功能

- 创建定时/重复提醒（支持分钟、小时、天）
- 系统原生通知 + 窗口弹窗双模式提醒
- 系统托盘常驻，双击打开窗口，右键快捷菜单
- 支持日期范围、每日活跃时段、重复次数限制
- TOML 格式持久化存储

## 快速开始

```powershell
cargo run
```

## 开发

```powershell
cargo check          # 编译检查
cargo test           # 运行测试
cargo test <name>    # 运行单个测试
```

## 技术栈

- **语言**：Rust
- **GUI**：Slint
- **通知**：notify-rust
- **托盘**：tray-icon + muda
- **存储**：TOML（serde + toml）

## 项目结构

```
src/
├── lib.rs           # 库 crate
├── main.rs          # 应用入口
├── reminder.rs      # 提醒模型 + 存储
├── store.rs         # 状态管理（CRUD + 调度）
├── config.rs        # 配置管理
├── time.rs          # 时间解析 + 触发判断
├── ui.rs            # UI 数据转换
├── notification.rs  # 系统通知
├── tray.rs          # 系统托盘
└── logging.rs       # 日志系统
ui/
└── main.slint       # GUI 定义
```

## 存储位置

- 数据：`%APPDATA%\due\reminders.toml`
- 配置：`%APPDATA%\due\config.toml`

## 许可证

MIT
