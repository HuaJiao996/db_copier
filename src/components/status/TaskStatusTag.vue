<template>
  <el-tag :type="tagType">{{ statusText }}</el-tag>
</template>

<script setup lang="ts">
import type { TaskStatus } from '@/types'
import { computed } from 'vue'

const props = defineProps<{
  status: TaskStatus
}>()

const tagType = computed(() => {
  switch (props.status.status) {
    case 'pending':
      return 'info'
    case 'running':
      return 'warning'
    case 'completed':
      return 'success'
    case 'failed':
      return 'danger'
    default:
      return 'info'
  }
})

const statusText = computed(() => {
  switch (props.status.status) {
    case 'pending':
      return '等待中'
    case 'running':
      return '运行中'
    case 'completed':
      return '已完成'
    case 'failed':
      return '失败'
    default:
      return '未知'
  }
})
</script> 