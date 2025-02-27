<template>
  <el-dropdown @command="handleCommand" class="language-switcher">
    <span class="el-dropdown-link">
      {{ currentLanguage }}
      <el-icon class="el-icon--right">
        <arrow-down />
      </el-icon>
    </span>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="zh-CN">中文</el-dropdown-item>
        <el-dropdown-item command="en-US">English</el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ArrowDown } from '@element-plus/icons-vue'

const { locale } = useI18n()

const currentLanguage = computed(() => {
  return locale.value === 'zh-CN' ? '中文' : 'English'
})

const handleCommand = (command: string) => {
  locale.value = command
}
</script>

<style scoped>
.language-switcher {
  margin-left: auto;
  padding: 0 20px;
  cursor: pointer;
}

.el-dropdown-link {
  display: flex;
  align-items: center;
  color: var(--el-text-color-primary);
}
</style> 