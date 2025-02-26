<script setup lang="ts">
import { watch } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from 'element-plus';
import { Setting, Plus } from '@element-plus/icons-vue';

const props = defineProps<{
  loading: boolean;
  configs: string[];
  currentConfig: string;
}>();

const emit = defineEmits<{
  'create': [],
  'select': [name: string],
  'delete': [name: string],
}>();

const handleSelect = (index: string) => {
  console.log('选择配置:', index);
  emit('select', index);
};

// 监听 currentConfig 的变化
watch(() => props.currentConfig, (newVal: string) => {
  console.log('当前选中配置:', newVal);
});

const deleteConfig = async (name: string) => {
  try {
    await ElMessageBox.confirm('确定要删除该配置吗？', '提示', {
      type: 'warning',
    });
    await invoke('delete_config', { name });
    ElMessage.success('删除配置成功');
    emit('delete', name);
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除配置失败: ' + error);
    }
  }
};
</script>

<template>
  <el-card class="config-list" v-loading="loading">
    <template #header>
      <div class="card-header">
        <span>配置列表</span>
      </div>
    </template>

    <el-menu
      :default-active="currentConfig"
      class="config-menu"
      :router="false"
      :unique-opened="true"
      @select="handleSelect"
    >
      <el-menu-item
        v-for="config in configs"
        :key="config"
        :index="config"
      >
        <template #default>
          <div class="config-menu-item">
            <div class="config-info">
              <el-icon><Setting /></el-icon>
              <span>{{ config }}</span>
            </div>
            <el-button
              link
              type="danger"
              @click.stop="deleteConfig(config)"
            >
              删除
            </el-button>
          </div>
        </template>
      </el-menu-item>
    </el-menu>

    <!-- 如果没有数据显示空状态 -->
    <el-empty
      v-if="configs.length === 0"
      description="暂无配置"
    >
      <el-button type="primary" @click="emit('create')">
        <el-icon><Plus /></el-icon>
        创建配置
      </el-button>
    </el-empty>
  </el-card>
</template>

<style scoped>
.config-list {
  margin-bottom: 20px;
}

.config-menu {
  border-right: none;
}

.config-menu-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.config-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

:deep(.el-menu-item) {
  height: 40px;
  line-height: 40px;
  padding: 0 16px;
}

:deep(.el-menu-item.is-active) {
  background-color: var(--el-menu-hover-bg-color);
}
</style> 