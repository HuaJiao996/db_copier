# 组件使用指南

## Element Plus 组件使用规范

本项目使用 Element Plus 作为 UI 组件库，并采用以下规范：

### 1. 组件标签使用驼峰命名法

在模板中使用 Element Plus 组件时，应使用驼峰命名法，例如：

```vue
<!-- 推荐 -->
<ElButton type="primary">按钮</ElButton>
<ElTable :data="tableData">
  <ElTableColumn prop="name" label="名称" />
</ElTable>

<!-- 不推荐 -->
<el-button type="primary">按钮</el-button>
<el-table :data="tableData">
  <el-table-column prop="name" label="名称" />
</el-table>
```

### 2. 导入方式

项目已配置 Element Plus 的自动导入功能，无需手动导入组件和常用API：

```vue
<script setup lang="ts">
// 无需导入，可以直接使用
// import { ElMessage, ElMessageBox } from 'element-plus'

// 直接使用
ElMessage.success('操作成功');
</script>
```

自动导入的组件和API包括：
- 所有Element Plus组件（如ElButton、ElTable等）
- 常用API（如ElMessage、ElMessageBox、ElNotification、ElLoading）

### 3. 基础组件

项目中封装了一些基于 Element Plus 的基础组件，位于 `src/components/common` 目录下：

- `BaseButton.vue`: 按钮组件
- `BaseCard.vue`: 卡片组件
- `BaseTable.vue`: 表格组件

这些组件提供了更符合项目需求的默认配置和样式，建议优先使用这些组件。

### 4. 图标使用

使用 Element Plus 图标时，需要先导入图标，然后在模板中使用：

```vue
<script setup lang="ts">
import { Search, Edit, Delete } from '@element-plus/icons-vue'
</script>

<template>
  <ElButton type="primary">
    <ElIcon><Search /></ElIcon>
    搜索
  </ElButton>
</template>
```

## 自定义组件

项目按功能将组件分为以下几类：

- `common`: 通用基础组件
- `config`: 配置相关组件
- `layout`: 布局相关组件
- `task`: 任务相关组件

创建新组件时，请将组件放在对应的目录中，并遵循以下命名规范：

1. 文件名使用 PascalCase 命名法，例如 `TableConfig.vue`
2. 组件名应具有描述性，表明组件的用途
3. 避免使用缩写或单个单词作为组件名 