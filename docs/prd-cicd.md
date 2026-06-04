# PRD: GitHub Actions CI/CD — 自动构建 Windows exe + 手动/每夜版本

## Problem Statement

due 项目目前没有任何 CI/CD 流水线。每次发版都需要开发者在本地手动执行 `cargo build --release`，然后手动打包、上传产物。这导致：

- 发版流程繁琐且容易出错
- 没有标准化的构建产物（exe）
- 其他协作者或用户无法方便地获取最新构建
- 没有自动化的每夜构建来追踪最新开发进度

## Solution

通过 GitHub Actions 建立 CI/CD 流水线，实现：

1. **CI 检查**：每次 PR / push 自动运行 `cargo check` + `cargo test`，保障代码质量
2. **手动构建**：通过 `workflow_dispatch` 手动触发，生成 exe 产物上传为 Artifact
3. **每夜构建**：每天自动构建最新代码，生成带日期标记的 prerelease Release
4. **Release 发布**：推送 tag 时自动构建并创建正式 GitHub Release

## User Stories

1. As a 开发者, I want 推送代码或创建 PR 时自动运行编译检查和测试, so that 我能及时发现代码问题
2. As a 开发者, I want 手动触发构建流水线, so that 我可以在需要时随时生成 Windows exe 产物
3. As a 开发者, I want 每天自动构建最新代码, so that 我能持续验证项目可编译性并获得最新的 nightly 构建
4. As a 用户, I want 从 GitHub Releases 下载预编译的 exe, so that 我不需要本地安装 Rust 工具链就能使用 due
5. As a 开发者, I want 推送 tag（如 `v0.2.0`）时自动创建 Release, so that 发版流程自动化且标准化
6. As a 开发者, I want 构建产物包含版本号和日期信息, so that 我能区分不同版本的构建
7. As a 开发者, I want 每夜构建使用 `nightly-YYYYMMDD` 格式标记, so that 我能清楚知道构建日期
8. As a 用户, I want 在 GitHub Releases 页面看到清晰的版本说明, so that 我了解每个版本的变化
9. As a 开发者, I want CI 流水线在 Windows runner 上运行, so that 构建环境与目标平台一致
10. As a 开发者, I want 构建失败时收到通知, so that 我能快速定位和修复问题
11. As a 开发者, I want 缓存 Cargo 依赖, so that CI 构建速度更快
12. As a 用户, I want 下载的 exe 是 release 模式编译的, so that 获得最佳性能

## Implementation Decisions

### 1. 工作流文件结构

创建 `.github/workflows/` 目录，包含两个工作流文件：

- **`ci.yml`** — CI 检查（`cargo check` + `cargo test`），触发条件：push 到 main、PR
- **`release.yml`** — 构建 + 打包 + 上传，三种触发方式共存：

| 触发方式 | 行为 |
|---|---|
| `push tags`（`v*`） | 构建 + 上传 Artifact + 创建正式 Release |
| `workflow_dispatch`（手动） | 构建 + 上传 Artifact（不创建 Release） |
| `schedule`（每夜 cron） | 构建 + 上传 Artifact + 创建 prerelease Release |

### 2. 构建目标

- 目标平台：`x86_64-pc-windows-msvc`（GitHub Windows runner 默认）
- 构建模式：`release`
- 产物：单个 `due.exe` 文件，打包为 zip

### 3. 版本标记策略

- **手动构建**：Artifact 名称包含 run ID，不创建 Release
- **每夜构建**：tag `nightly-YYYYMMDD`，产物 `due-nightly-YYYYMMDD-windows-x86_64.zip`
- **Release**：tag `vX.Y.Z`，产物 `due-vX.Y.Z-windows-x86_64.zip`，版本号从 tag 提取

### 4. 每夜构建调度

使用 cron 表达式 `0 2 * * *`（UTC，北京时间上午 10 点），无条件运行（不做"无新提交则跳过"判断）。仅默认分支触发。

### 5. 每夜构建清理

保留最近 3 个 nightly release，自动清理更早的。使用 `gh release list --tag 'nightly-*'` + 排序删除实现。

### 6. 依赖缓存

使用 `Swatinem/rust-cache` action 缓存 `~/.cargo/registry`、`~/.cargo/git`、`target/`，加速后续构建。

### 7. Release 创建

使用 `softprops/action-gh-release` action，从 tag 名提取版本号，自动生成 changelog（基于 commits since last tag）。直接发布，不做 Draft。

### 8. CI 检查

仅 `cargo check` + `cargo test`，不加 clippy。不配置分支保护规则。

### 9. 版本嵌入

不在 exe 中嵌入版本信息，文件名区分即可。后续再加。

## Testing Decisions

### CI 工作流验证

- **冒烟测试**：工作流文件本身的 YAML 语法正确性（GitHub 会自动校验）
- **构建验证**：`cargo check` 和 `cargo test` 作为 CI 检查的一部分
- **产物验证**：构建后检查 exe 文件存在且大小合理

### 验证方式

1. 创建 PR 后观察 CI 是否自动触发并通过
2. 手动触发 workflow_dispatch，检查 Artifact 是否正确上传
3. 推送 test tag，验证 Release 是否正确创建
4. 等待次日检查每夜构建是否自动执行

## Out of Scope

- **Linux / macOS 构建**：当前仅需 Windows exe
- **代码签名**：不涉及 exe 数字签名
- **自动安装程序**：不创建 MSI/NSIS 安装包
- **Docker 化构建**：不使用容器构建
- **自动发布到包管理器**：不涉及 winget / scoop / chocolatey
- **构建通知**：使用 GitHub 默认的邮件通知，不额外配置 Slack/Discord 等
- **clippy 静态分析**：不在 CI 中启用
- **版本信息嵌入**：不在 exe 中嵌入版本号
- **分支保护规则**：不配置

## Further Notes

- 所有依赖均为纯 Rust Windows API 绑定，无需额外安装 C/C++ 工具链或系统库
- Slint 的 `slint-build` 会在 `cargo build` 时自动编译 UI 文件，无需额外步骤
- 建议首次实现时先完成 CI + 手动构建，验证通过后再添加每夜构建和 Release 流水线
