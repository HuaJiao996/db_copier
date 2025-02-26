<template>
  <el-button
    :type="type"
    :size="size"
    :disabled="disabled || loading"
    :loading="loading"
    :icon="icon"
    @click="handleClick"
  >
    <slot></slot>
  </el-button>
</template>

<script setup lang="ts">

interface Props {
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'text';
  size?: 'large' | 'default' | 'small';
  disabled?: boolean;
  loading?: boolean;
  icon?: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: 'primary',
  size: 'default',
  disabled: false,
  loading: false,
  icon: ''
});

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();

const handleClick = (event: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', event);
  }
};
</script>

<style scoped>
/* 可以添加自定义样式 */
</style> 