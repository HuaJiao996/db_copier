<template>
  <el-container class="layout-container">
    <el-header height="60px" style="padding: 0; border-bottom: 1px solid var(--el-border-color-light)">
      <el-row align="middle" style="height: 100%">
        <el-col :span="12">
          <h2 style="margin: 0 0 0 20px">数据库复制工具</h2>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-space>
            <el-button 
              :type="route.path === '/' ? 'primary' : 'default'"
              @click="router.push('/')"
            >
              配置
            </el-button>
            <el-button 
              :type="route.path === '/tasks' ? 'primary' : 'default'"
              @click="router.push('/tasks')"
              style="margin-right: 20px"
            >
              任务列表
            </el-button>
          </el-space>
        </el-col>
      </el-row>
    </el-header>

    <el-main>
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()
</script>

<style>
.layout-container {
  height: 100vh;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style> 