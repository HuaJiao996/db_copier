# PostgreSQL 数据库复制工具 [WIP]

这是一个由AI辅助开发的PostgreSQL数据库复制工具，基于Tauri和Vue构建，支持通过SSH隧道连接远程数据库，并提供数据脱敏功能。提供现代化的桌面应用界面进行配置和操作。

> **注意**: 本项目目前处于开发中 (Work In Progress)，部分功能可能不完整或存在变更。

## 功能特点

- 基于Tauri构建的跨平台桌面应用
- 现代化的Vue 3用户界面
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
   - Node.js 16+
   - Rust 1.69+
   - 系统依赖（参见 [Tauri 前置要求](https://tauri.app/v1/guides/getting-started/prerequisites)）

2. 克隆仓库：
```bash
git clone [repository_url]
cd dbcopy
```

3. 安装依赖：
```bash
# 安装前端依赖
npm install

# 安装 Rust 依赖（自动完成）
```

4. 开发模式运行：
```bash
npm run tauri dev
```

5. 构建发布版本：
```bash
npm run tauri build
```

## 配置说明

通过应用界面可以方便地配置以下内容：

1. SSH连接信息：
   - 主机地址
   - 端口
   - 用户名
   - 密钥文件路径

2. 源数据库配置：
   - 主机
   - 端口
   - 数据库名
   - 用户名
   - 密码

3. 目标数据库配置：
   - 主机
   - 端口
   - 数据库名
   - 用户名
   - 密码

4. 表配置：
   - 可以手动添加表
   - 可以从数据库获取表列表
   - 支持配置要复制的列
   - 支持配置数据脱敏规则

配置示例：
```yaml
ssh:
  host: remote.example.com
  port: 22
  username: your_username
  # 使用密钥认证（推荐）
  private_key_path: ~/.ssh/id_rsa
  # 或使用密码认证（不推荐）
  # password: your_password

source_db:
  host: localhost  # 通过SSH隧道访问的数据库主机
  port: 5432
  database: source_db_name
  username: db_user
  password: db_password

target_db:
  host: localhost
  port: 5432
  database: target_db_name
  username: db_user
  password: db_password

# 要复制的表配置
tables:
  - name: users
    # 如果为空则复制所有列
    columns: []
    # 脱敏规则
    mask_rules:
      - column: email
        method: partial  # 可选: partial, hash, random
      - column: phone
        method: partial
  - name: orders
    columns: [id, order_number, created_at, status]
    mask_rules:
      - column: order_number
        method: hash

## 脱敏方法说明

1. `partial`: 保留首尾字符，中间用星号代替
   - 例如：`john@example.com` -> `j***@example.com`

2. `hash`: 使用MD5哈希替换整个值
   - 例如：`12345` -> `827ccb0eea8a706c4c34a16891f84e7b`

3. `random`: 保持原有长度，但用随机字符替换
   - 例如：`secret` -> `x7k9p2`

## 开发相关

- 前端技术栈：
  - Vue 3
  - TypeScript
  - Vite
  - Element Plus
  - Vue Router
  - Pinia

- 后端技术栈：
  - Rust
  - Tauri 2.0
  - tokio-postgres
  - ssh2

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