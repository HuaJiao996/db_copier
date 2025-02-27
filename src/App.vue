<script setup lang="ts">
import { RouterView } from "vue-router";
import { ElConfigProvider } from "element-plus";
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import enUs from 'element-plus/es/locale/lang/en'
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import LanguageSwitcher from './components/LanguageSwitcher.vue';
import {
  Monitor,
  Setting,
} from '@element-plus/icons-vue';

// 使用i18n
const { locale } = useI18n();

// 根据当前语言选择Element Plus的语言包
const elementLocale = computed(() => {
  return locale.value === 'zh-CN' ? zhCn : enUs;
});
</script>

<template>
  <el-config-provider :locale="elementLocale">
    <div class="app-wrapper">
      <!-- 顶部导航栏 -->
       <div class="top-menu">
          <el-menu
            mode="horizontal"
            :router="true"
            class="top-menu-container"
          >
          <el-menu-item index="1" route="/">
            <el-icon><Monitor /></el-icon>
            <span>{{ $t('nav.taskManager') }}</span>
          </el-menu-item>
          <el-menu-item index="2" route="/config">
            <el-icon><Setting /></el-icon>
            <span>{{ $t('nav.configManager') }}</span>
          </el-menu-item>
          </el-menu>
          <LanguageSwitcher />
      </div>
      
      <!-- 主要内容区 -->
      <el-container class="main-container">
        <el-main class="main-content">
          <router-view></router-view>
        </el-main>
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
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.top-menu-container {
  width: 100%;
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

.text-success {
  color: var(--el-color-success);
}

.text-danger {
  color: var(--el-color-danger);
}
</style>