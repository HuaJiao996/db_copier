# PostgreSQL 数据库复制工具 [WIP]

这是一个PostgreSQL数据库复制工具，基于Tauri和Vue构建，支持通过SSH隧道连接远程数据库，并提供数据脱敏功能。提供现代化的桌面应用界面进行配置和操作。

> **注意**: 本项目目前处于开发中 (Work In Progress)，部分功能可能不完整或存在变更。

## 功能特点

- 基于Tauri 2.0构建的跨平台桌面应用
- 现代化的Vue 3用户界面，使用Element Plus组件库
- 支持通过SSH隧道连接远程PostgreSQL数据库
- 支持选择性复制表和列
- 提供多种数据脱敏方法：
  - 部分掩码（保留首尾字符）
  - 哈希掩码
  - 随机替换
- 详细的操作日志记录
- 使用配置文件管理连接信息和脱敏规则
- 任务管理功能：
  - 实时监控任务进度
  - 支持批量启动任务
  - 任务状态筛选和搜索
  - 自动刷新运行中的任务状态
  - 详细的任务执行记录

## 安装

### 从发布版本安装

访问 [Releases](https://github.com/HuaJiao996/db_copier/releases) 页面下载适用于您操作系统的安装包：

- Windows: `.msi` 或 `.exe`
- macOS: `.dmg`
- Linux: `.deb` 或 `.AppImage`

### 从源码构建

1. 确保已安装以下依赖：
   - Node.js 18+
   - Rust 1.75+
   - pnpm 8+
   - 系统依赖（参见 [Tauri 前置要求](https://tauri.app/v1/guides/getting-started/prerequisites)）

2. 克隆仓库：
```bash
git clone https://github.com/HuaJiao996/db_copier.git
cd db_copier
```

3. 安装依赖：
```bash
# 安装前端依赖
pnpm install

# 安装 Rust 依赖（自动完成）
```

4. 开发模式运行：
```bash
pnpm tauri dev
```

5. 构建发布版本：
```bash
pnpm tauri build
```

## 技术栈

### 前端技术栈
- Vue 3.5.13
- TypeScript 5.7.3
- Vite 6.1.1
- Element Plus 2.4.4
- Vue Router 4.2.5
- Pinia 3.0.1
- @vueuse/core 12.7.0

### 后端技术栈
- Rust 1.75+
- Tauri 2.2.7
- tokio-postgres 0.7.13
- ssh2 0.9
- tokio 1.36 (异步运行时)
- rusqlite 0.30.0 (本地数据存储)

## 贡献指南

1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的改动 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建一个 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 注意事项

1. 请确保SSH密钥或密码配置正确
2. 建议使用SSH密钥认证而不是密码认证
3. 目标数据库需要有足够的权限创建表和写入数据
4. 建议在执行大规模数据复制前先进行小规模测试
5. 所有密码和敏感信息都经过加密存储
6. 应用日志保存在系统用户目录下的应用数据文件夹中

## 使用说明

### 配置管理

1. 在配置管理页面可以创建、编辑、复制和删除配置
2. 每个配置包含源数据库、目标数据库和表配置信息
3. 支持SSH隧道连接远程数据库
4. 可以选择要复制的表和列，并配置数据脱敏规则

### 任务管理

1. 任务监控：
   - 实时显示任务执行进度
   - 自动刷新运行中的任务状态（每5秒）
   - 显示当前运行中的任务数量

2. 任务筛选：
   - 按状态筛选（运行中、等待中、已完成、失败）
   - 支持任务ID和消息内容搜索
   - 分页显示任务列表

3. 任务操作：
   - 支持单个任务启动
   - 支持批量任务启动
   - 查看任务详细信息
   - 手动刷新任务状态

4. 任务状态说明：
   - 等待中：任务已创建但尚未开始执行
   - 运行中：任务正在执行
   - 已完成：任务成功完成
   - 失败：任务执行失败，可查看错误信息

## 系统要求

- **Windows**: Windows 10/11 64位
- **macOS**: macOS 11.0+ (Big Sur及以上)
- **Linux**: 支持Debian/Ubuntu、Fedora、Arch等主流发行版

## 常见问题

1. **Q: 如何解决SSH连接失败的问题？**  
   A: 请检查SSH主机地址、端口、用户名和密钥文件路径是否正确。确保目标服务器允许SSH连接，并且您的密钥有正确的权限。

2. **Q: 数据库连接失败怎么办？**  
   A: 检查数据库主机、端口、用户名和密码是否正确。如果使用SSH隧道，确保SSH连接正常。

3. **Q: 应用崩溃或无响应怎么办？**  
   A: 请查看应用日志文件，位于系统用户目录下的应用数据文件夹中。如果问题持续，请在GitHub上提交issue。 
