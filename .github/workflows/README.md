# GitHub Actions 工作流文档

## 概述

WinCleaner 使用 GitHub Actions 实现自动化构建和发布流程。包含两个主要工作流：

## 工作流说明

### 1. CI 工作流 (`ci.yml`)

**触发条件：**
- 推送到 `main` 分支
- 创建 Pull Request

**功能：**
- ✅ 代码格式检查 (`cargo fmt`)
- ✅ 静态分析 (`cargo clippy`)
- ✅ 构建测试
- ✅ 单元测试运行
- ✅ 发布版本构建验证

### 2. 发布工作流 (`release.yml`)

**触发条件：**
- 推送到 `main` 分支
- 手动触发 (`workflow_dispatch`)

**功能：**
- ✅ 自动生成版本标签 (格式: `v2024.10.02.1`)
- ✅ Windows x64 构建
- ✅ 自动创建 GitHub Release
- ✅ 构建产物缓存优化
- ✅ ZIP 压缩包生成

## 使用方法

### 自动发布

1. 推送代码到 `main` 分支
2. GitHub Actions 会自动：
   - 生成新的版本标签
   - 构建 Windows 版本
   - 创建 Release 并上传构建产物

### 手动发布

1. 进入 GitHub 仓库的 Actions 页面
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow" → "Run workflow"
4. 等待构建完成

### 下载构建产物

构建完成后，可以在以下位置找到构建产物：

1. **GitHub Release:** 自动创建的 Release 页面
2. **Actions Artifacts:** 工作流运行页面的 Artifacts 部分

## 版本标签格式

版本标签使用日期格式：`vYYYY.MM.DD.i`

- `YYYY`：年份
- `MM`：月份
- `DD`：日期
- `i`：当日第几次构建（从1开始）

例如：`v2024.10.02.1`

## 构建产物结构

```
wincleaner-windows-x64.zip
├── wincleaner.exe      # 主程序
├── assets/             # 图标等资源文件
├── README.md           # 项目文档
└── LICENSE             # 许可证文件
```

## 故障排查

### 构建失败

1. 检查 Cargo.toml 依赖是否正确
2. 查看 Actions 日志中的错误信息
3. 确保代码能在本地正常构建

### Release 创建失败

1. 检查 GitHub Token 权限
2. 确认标签生成是否正常
3. 查看是否有同名 Release 已存在

### 缓存问题

如果遇到奇怪的构建错误，可以尝试清除缓存：
- 进入 Actions 页面
- 找到对应的 workflow run
- 点击 "Re-run jobs" → "Re-run failed jobs"

## 性能优化

- **缓存策略：** 使用 `actions/cache` 缓存 Cargo 依赖和构建产物
- **并行构建：** 不同平台可以并行构建
- **增量构建：** 只重新构建变更的部分

## 安全考虑

- 使用最小权限原则
- 不存储敏感信息在代码中
- 使用 GitHub Secrets 管理密钥（如需要）

## 扩展建议

如果需要添加更多功能：

1. **多平台支持：** 添加 Linux 和 macOS 构建
2. **代码覆盖率：** 集成 codecov
3. **依赖扫描：** 添加安全扫描
4. **性能测试：** 添加基准测试
5. **文档生成：** 自动生成 API 文档