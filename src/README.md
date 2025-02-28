# 前端目录结构

本项目采用模块化的目录结构，按功能和类型分类组织代码，提高可维护性和可扩展性。

## 目录结构说明

```
src/
├── api/                # API请求相关
├── assets/             # 静态资源文件
├── components/         # 组件
│   ├── common/         # 通用基础组件
│   ├── config/         # 配置相关组件
│   ├── layout/         # 布局相关组件
│   └── task/           # 任务相关组件
├── constants/          # 常量定义
├── hooks/              # 自定义钩子函数
├── layouts/            # 页面布局
├── pages/              # 页面组件
├── services/           # 服务层，处理API调用
├── store/              # 状态管理
├── styles/             # 样式文件
├── utils/              # 工具函数
├── App.vue             # 应用入口组件
├── main.ts             # 应用入口文件
├── router.ts           # 路由配置
├── types.ts            # 类型定义
└── vite-env.d.ts       # Vite环境类型声明
```

## 各目录职责

- **api**: 存放API请求相关的代码，如请求拦截器、响应处理等
- **assets**: 存放静态资源文件，如图片、字体等
- **components**: 按功能分类的组件
  - **common**: 通用基础组件，如按钮、表格、卡片等
  - **config**: 配置相关组件，如配置表单、配置列表等
  - **layout**: 布局相关组件，如页面布局、侧边栏等
  - **task**: 任务相关组件，如任务状态标签、任务列表等
- **constants**: 存放应用中使用的常量，如枚举值、配置项等
- **hooks**: 自定义钩子函数，如加载状态、通知等
- **layouts**: 页面布局组件，如默认布局等
- **pages**: 页面级组件，对应路由
- **services**: 服务层，处理API调用和数据处理
- **store**: 状态管理，集中管理应用状态
- **styles**: 样式文件，如全局样式、主题等
- **utils**: 工具函数，如本地存储、错误处理等

## 导入规范

为了保持代码的一致性和可维护性，请遵循以下导入规范：

1. 使用相对路径导入组件和模块
2. 使用索引文件（index.ts）导出模块内容
3. 按照以下顺序组织导入语句：
   - Vue相关导入
   - 第三方库导入
   - 本地组件和模块导入
   - 类型导入

示例：

```typescript
// 1. Vue相关导入
import { ref, computed, onMounted } from 'vue'

// 2. 第三方库导入
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

// 3. 本地组件和模块导入
import { useLoading, useNotification } from '@/composables'
import { configApi } from '@/services'
import BaseTable from '@/components/common/BaseTable.vue'

// 4. 类型导入
import type { Config, Task } from '@/types'
``` 