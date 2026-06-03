## Parent

#1 - feat: 桌面提醒应用核心功能

## What to build

搭建项目基础结构，定义提醒数据模型，实现 TOML 格式的持久化存储。

核心内容：
- 添加项目依赖（slint, serde, toml, chrono, notify-rust, tokio）
- 定义 Reminder 结构体，包含标题、内容、时间设置、重复规则、状态等字段
- 实现 TOML 格式的存储/加载功能
- 存储位置：%APPDATA%\due\ 或 %LOCALAPPDATA%\due\

## Acceptance criteria

- [x] 项目可编译运行，依赖正确配置
- [x] Reminder 结构体支持所有必要字段（标题、内容、时间、状态）
- [x] 可以创建 Reminder 并序列化为 TOML
- [x] 可以从 TOML 文件加载 Reminder 列表
- [x] 存储路径使用标准应用数据目录

## Blocked by

None - 可以立即开始