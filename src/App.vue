<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElConfigProvider } from 'element-plus'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import {
  Monitor,
  Setting,
} from '@element-plus/icons-vue';

const memoryUsage = ref<number>(0);
let memoryUpdateInterval: number | null = null;

const updateMemoryUsage = async () => {
  try {
    const usage = await invoke<number>('get_memory_usage');
    memoryUsage.value = usage;
  } catch (error) {
    console.error('获取内存使用量失败:', error);
  }
};

onMounted(async () => {
  updateMemoryUsage();
  memoryUpdateInterval = window.setInterval(updateMemoryUsage, 5000);
});

onUnmounted(() => {
  if (memoryUpdateInterval) {
    clearInterval(memoryUpdateInterval);
  }
});
</script>

<template>
  <el-config-provider :locale="zhCn">
    <div class="app-wrapper">
      <!-- 顶部导航栏 -->
      <el-menu
        mode="horizontal"
        :router="true"
        class="top-menu"
      >
        <el-menu-item index="1" route="/">
          <el-icon><Monitor /></el-icon>
          <span>任务管理</span>
        </el-menu-item>
        <el-menu-item index="2" route="/config">
          <el-icon><Setting /></el-icon>
          <span>配置管理</span>
        </el-menu-item>
      </el-menu>

      <!-- 主要内容区 -->
      <el-container class="main-container">
        <el-main class="main-content">
          <router-view></router-view>
        </el-main>
        <!-- 状态栏 -->
        <el-footer height="30px" class="status-bar">
          <div class="status-item">
            <el-icon><Monitor /></el-icon>
            <span>内存使用: {{ (memoryUsage / 1024 / 1024).toFixed(2) }} MB</span>
          </div>
        </el-footer>
      </el-container>
    </div>
  </el-config-provider>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: #2c3e50;
  background-color: #f6f6f6;
  
  /* 禁用文本选择 */
  user-select: none;
  -webkit-user-select: none;
}

body {
  margin: 0;
  padding: 0;
}

/* 自定义滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #555;
}
</style>

<style scoped>
.app-wrapper {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f5f7fa;
}

.top-menu {
  border-bottom: 1px solid var(--el-border-color-light);
  padding: 0 20px;
}

.main-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.main-content {
  flex: 1;
  overflow: auto;
  padding: 0;
  background-color: transparent;
}

.status-bar {
  height: 28px;
  background-color: var(--el-bg-color);
  border-top: 1px solid var(--el-border-color-lighter);
  padding: 0 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.status-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-item .el-icon {
  font-size: 14px;
}

.text-success {
  color: var(--el-color-success);
}

.text-danger {
  color: var(--el-color-danger);
}
</style>