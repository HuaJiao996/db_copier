<template>
  <el-card
    :class="['base-card', shadow ? 'is-' + shadow : '']"
    :body-style="bodyStyle"
  >
    <template v-if="$slots.header" #header>
      <div class="card-header">
        <slot name="header"></slot>
      </div>
    </template>
    <div class="card-body">
      <slot></slot>
    </div>
  </el-card>
</template>

<script setup lang="ts">
interface Props {
  shadow?: 'always' | 'hover' | 'never';
  bodyStyle?: Record<string, string>;
  title?: string;
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  shadow: 'always',
  bodyStyle: () => ({}),
  title: '',
  loading: false
});
</script>

<style scoped>
.base-card {
  margin-bottom: 16px;
  border-radius: 8px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-body {
  padding: 8px 0;
}
</style> 