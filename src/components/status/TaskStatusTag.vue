<template>
  <el-tag :type="tagType">{{ statusText }}</el-tag>
</template>

<script setup lang="ts">
import type { TaskStatus } from '@/types'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  status: TaskStatus
}>()

const { t } = useI18n()

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
  return t(`taskManager.status.${props.status.status}`)
})
</script> 