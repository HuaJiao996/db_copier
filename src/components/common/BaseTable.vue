<template>
  <div class="base-table-wrapper">
    <div v-if="$slots.header" class="table-header">
      <slot name="header"></slot>
    </div>
    
    <el-table
      v-bind="$attrs"
      :data="data"
      :border="border"
      :stripe="stripe"
      :height="height"
      :max-height="maxHeight"
      :row-key="rowKey"
      :loading="loading"
      @row-click="handleRowClick"
      @selection-change="handleSelectionChange"
    >
      <el-table-column
        v-if="showSelection"
        type="selection"
        width="55"
        align="center"
      />
      
      <slot></slot>
      
      <el-table-column
        v-if="showActions && actions.length > 0"
        :label="actionsLabel"
        :width="actionsWidth"
        :fixed="actionsFixed"
        align="center"
      >
        <template #default="scope">
          <el-button-group>
            <template v-for="(action, index) in actions" :key="index">
              <el-button
                v-if="!action.show || action.show(scope.row)"
                :type="action.type || 'primary'"
                :size="action.size || 'small'"
                :disabled="action.disabled ? action.disabled(scope.row) : false"
                :link="action.link"
                @click.stop="handleAction(action, scope.row, scope.$index)"
              >
                <el-icon v-if="action.icon">
                  <component :is="action.icon" />
                </el-icon>
                {{ action.label }}
              </el-button>
            </template>
          </el-button-group>
        </template>
      </el-table-column>
    </el-table>
    
    <div v-if="showPagination" class="pagination-container">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="pageSizes"
        :total="total"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

export interface TableAction<T = any> {
  label: string;
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'text';
  size?: 'large' | 'default' | 'small';
  icon?: string;
  link?: boolean;
  show?: (row: T) => boolean;
  disabled?: (row: T) => boolean;
  handler: (row: T, index: number) => void;
}

interface Props<T = any> {
  data: T[];
  loading?: boolean;
  border?: boolean;
  stripe?: boolean;
  height?: string | number;
  maxHeight?: string | number;
  rowKey?: string;
  showSelection?: boolean;
  showActions?: boolean;
  actions?: TableAction<T>[];
  actionsLabel?: string;
  actionsWidth?: string | number;
  actionsFixed?: boolean | 'left' | 'right';
  showPagination?: boolean;
  total?: number;
  defaultPageSize?: number;
  pageSizes?: number[];
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  border: true,
  stripe: true,
  showSelection: false,
  showActions: false,
  actions: () => [],
  actionsLabel: '操作',
  actionsWidth: 150,
  actionsFixed: 'right',
  showPagination: false,
  total: 0,
  defaultPageSize: 10,
  pageSizes: () => [10, 20, 50, 100],
});

const emit = defineEmits<{
  (e: 'row-click', row: any, column: any, event: Event): void;
  (e: 'selection-change', selection: any[]): void;
  (e: 'page-change', page: number): void;
  (e: 'size-change', size: number): void;
  (e: 'action', action: string, row: any, index: number): void;
}>();

const currentPage = ref(1);
const pageSize = ref(props.defaultPageSize);

const handleRowClick = (row: any, column: any, event: Event) => {
  emit('row-click', row, column, event);
};

const handleSelectionChange = (selection: any[]) => {
  emit('selection-change', selection);
};

const handleSizeChange = (size: number) => {
  pageSize.value = size;
  emit('size-change', size);
};

const handleCurrentChange = (page: number) => {
  currentPage.value = page;
  emit('page-change', page);
};

const handleAction = (action: TableAction, row: any, index: number) => {
  action.handler(row, index);
  emit('action', action.label, row, index);
};
</script>

<style scoped>
.base-table-wrapper {
  width: 100%;
}

.table-header {
  margin-bottom: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.pagination-container {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.empty-data {
  padding: 32px 0;
}
</style> 