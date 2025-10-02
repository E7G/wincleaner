# WinCleaner 🧹

一个现代化的Windows系统清理工具，基于Rust和Freya GUI库开发。提供直观的图形界面，帮助用户安全地清理系统垃圾文件。

![License](https://img.shields.io/badge/license-GPL3-blue.svg)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/platform-windows-lightgrey)

## ✨ 功能特点

- **🎯 分类清理**：开发工具缓存、应用缓存、系统清理三大类别
- **🔒 安全确认**：重要文件清理前需要用户二次确认
- **🎨 现代化界面**：Apple设计风格，支持深色/浅色主题切换
- **📊 实时反馈**：显示清理进度和结果通知
- **⚡ 批量模式**：支持同时清理多个项目
- **🛡️ 智能检测**：自动检查目标路径是否存在，避免误操作

## 🚀 快速开始

### 系统要求
- Windows 10/11
- Rust 1.70+ (用于编译)
- 管理员权限（部分清理功能需要）

### 安装运行

```bash
# 克隆仓库
git clone https://github.com/e7g/wincleaner.git
cd wincleaner

# 编译运行
cargo run --release
```

### 使用说明

1. **选择清理类别**：点击左侧分类（开发工具、应用缓存、系统清理）
2. **选择清理项目**：在右侧列表中选择要清理的项目
3. **执行清理**：点击"清理"按钮，重要操作会弹出确认对话框
4. **批量清理**：开启"批量模式"可同时选择多个项目

## 🧹 支持的清理项目

### 开发工具缓存
| 项目 | 描述 | 预估大小 |
|------|------|----------|
| Go模块缓存 | 清理Go语言模块缓存 | ~1-3GB |
| Gradle缓存 | 清理Gradle构建缓存 | ~500MB-2GB |
| Cargo缓存 | 清理Rust包管理器缓存 | ~2GB |
| npm缓存 | 清理Node.js包缓存 | ~200MB |

### 应用缓存
| 项目 | 描述 | 预估大小 |
|------|------|----------|
| Trae AI聊天记录 | 清理AI助手聊天记录 | ~100MB-1GB |
| 酷狗音乐图片缓存 | 清理音乐应用图片缓存 | ~500MB |
| VSCode Cpptools缓存 | 清理VSCode C++扩展缓存 | ~1GB |
| Office更新缓存 | 清理Office更新残留文件 | ~2GB |

### 系统清理
| 项目 | 描述 | 预估大小 | 权限要求 |
|------|------|----------|----------|
| 系统组件清理 | 清理Windows更新组件 | ~1-3GB | 需要管理员 |
| 磁盘清理 | 运行Windows磁盘清理工具 | 可变 | 标准用户 |
| 清空回收站 | 永久删除回收站内容 | 可变 | 标准用户 |

## 🛡️ 安全机制

- **🔍 路径验证**：清理前自动检查目标路径是否存在
- **⚠️ 危险警告**：对可能影响系统稳定性的操作进行特别标识
- **🔄 确认对话框**：重要操作需要用户二次确认
- **📋 操作日志**：所有清理操作都有详细的错误处理和反馈

## 🎨 界面预览

### 主界面
![主界面](screenshots/屏幕截图%202025-10-02%20223916.png)

### 浅色主题
![浅色主题](screenshots/屏幕截图%202025-10-02%20223951.png)

### 批量模式
![批量模式](screenshots/屏幕截图%202025-10-02%20224147.png)

## 🏗️ 技术架构

### 核心技术栈
- **[Rust](https://www.rust-lang.org/)**：高性能系统编程语言
- **[Freya](https://freyaui.dev/)**：基于skia的跨平台GUI框架
- **[Tokio](https://tokio.rs/)**：异步运行时，提供卓越的性能

### 设计理念
- **响应式布局**：适配不同屏幕尺寸
- **主题系统**：支持深色/浅色主题动态切换
- **组件化架构**：可复用的UI组件设计
- **安全第一**：所有危险操作都有多重确认机制

## 🔧 开发指南

### 环境搭建
```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/e7g/wincleaner.git
cd wincleaner
```

### 项目结构
```
wincleaner/
├── src/
│   └── main.rs          # 主程序文件
├── Cargo.toml           # 项目依赖
├── README.md           # 项目文档
└── LICENSE             # GPL3许可证
```

### 构建发布
```bash
# 开发模式
cargo run

# 发布版本
cargo build --release
```

## 🤝 贡献指南

欢迎提交Issue和Pull Request！在贡献代码前，请：

1. 阅读并遵守[Rust编码规范](https://doc.rust-lang.org/1.0.0/style/README.html)
2. 确保所有测试通过
3. 更新相关文档
4. 遵循项目的安全设计原则

### 开发计划
- [ ] 添加更多清理项目
- [ ] 支持自定义清理路径
- [ ] 清理结果统计报告
- [ ] 多语言支持
- [ ] 自动更新功能

## 📄 许可证

本项目采用 [GNU General Public License v3.0](LICENSE) 许可证。

## ⚠️ 免责声明

本工具用于清理系统垃圾文件，使用时请谨慎操作：
- 建议在清理前备份重要数据
- 某些操作不可撤销，请仔细确认
- 开发者不对数据丢失承担责任
- 建议在了解清理内容后使用

## 🙏 致谢

感谢以下开源项目的贡献：
- [Freya](https://github.com/marc2332/freya) - 提供优秀的GUI框架
- [Tokio](https://github.com/tokio-rs/tokio) - 异步运行时

---

**WinCleaner** - 让Windows清理变得简单安全 🧹✨