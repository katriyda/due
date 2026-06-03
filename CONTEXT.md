# Project: due

## 术语表

### 提醒（Reminder）
应用的核心实体，包含时间规则和通知内容。

#### 提醒类型
- **定时提醒（One-time Reminder）**：在指定时间点触发一次
- **重复提醒（Recurring Reminder）**：按固定间隔重复触发，支持时间区间限制

#### 时间区间
- **日期范围（Date Range）**：提醒的起止日期，可选（不设则永久）
- **每日活跃时段（Daily Active Window）**：每天的活跃时间窗口，可选（不设则全天）

#### 提醒属性
- **标题（Title）**：提醒的简短描述，必填
- **内容（Content）**：详细提醒文本，可选
- **时间设置（Time Setting）**：本地时间或相对时间
- **重复次数限制（Repeat Limit）**：可选，达到次数后自动停止
- **完成标记（Completed）**：用户标记为已完成
- **启用状态（Enabled）**：启用/禁用

### 通知（Notification）
提醒触发时的用户通知方式。

#### 通知模式
- **系统通知（System Notification）**：Windows 原生通知中心
- **窗口弹窗（Window Popup）**：应用主窗口弹出提醒内容

#### 用户操作
- **关闭（Dismiss）**：关闭当前提醒弹窗
- **延后（Snooze）**：推迟指定时间后再提醒
- **标记完成（Mark Complete）**：标记为已完成

### 运行模式
- **后台托盘（System Tray）**：应用启动后最小化到系统托盘
- **GUI 配置界面（Configuration UI）**：通过主窗口配置提醒

### 存储
- **配置文件（Config）**：应用配置，TOML 格式
- **提醒数据（Reminders）**：所有提醒数据，TOML 格式
- **标准应用数据目录（App Data Directory）**：Windows `%APPDATA%\due\` 或 `%LOCALAPPDATA%\due\`

### 日志（Logging）
- **日志（Log）**：应用运行时的事件记录，用于调试和问题追踪
- **日志级别（Log Level）**：日志的严重程度
  - **error**：致命错误，操作失败且无法恢复
  - **warn**：可恢复错误，操作失败但有降级方案
  - **info**：关键业务事件，如提醒触发、添加、删除
  - **debug**：调试信息，如时间计算、数据加载详情
  - **trace**：详细追踪，如每次状态变化
- **日志文件（Log File）**：持久化存储的日志记录，位于 `%APPDATA%\due\logs\due.log`