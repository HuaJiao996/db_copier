<template>
  <el-form-item label="选择表和列" required>
    <div class="table-header">
      <div class="table-actions">
        <el-button 
          type="primary" 
          size="small" 
          @click="loadTables"
          :loading="tableLoading"
        >
          <el-icon><Refresh /></el-icon>
          刷新表列表
        </el-button>
        <el-button 
          type="primary" 
          size="small" 
          @click="saveTableConfig"
          :loading="loading"
        >
          <el-icon><Check /></el-icon>
          保存配置
        </el-button>
      </div>
    </div>

    <el-table
      :data="availableTables.map(name => ({ name }))"
      style="width: 100%"
      v-loading="tableLoading"
      row-key="name"
      @select="handleTableSelect"
      @expand-change="handleExpandChange"
    >
      <el-table-column type="expand">
          <template #default="{ row }">
          <div class="expanded-content">
            <div class="table-options">
              <el-checkbox 
                v-model="tableOptions[row.name].structureOnly"
                @change="() => handleTableOptionChange(row.name)"
              >
                仅复制表结构
              </el-checkbox>
              <el-checkbox 
                v-model="tableOptions[row.name].ignoreForeignKeys"
                @change="() => handleTableOptionChange(row.name)"
              >
                忽略外键关联
              </el-checkbox>
            </div>
            <div class="column-header">
              <span>列配置</span>
              <el-button 
                type="primary" 
                size="small" 
                link
                @click="refreshTableColumns(row.name)"
                :loading="columnLoading[row.name]"
              >
                <el-icon><Refresh /></el-icon>
                刷新列
              </el-button>
            </div>
            <el-table
              v-if="tableColumns[row.name]"
              :data="tableColumns[row.name].map(col => ({
                name: col,
                selected: selectedTableColumns[row.name]?.includes(col),
                maskRule: maskRules[row.name]?.find(rule => rule.column === col) || { column: col, rule_type: '', pattern: '' }
              }))"
              style="width: 100%"
              size="small"
              border
              :row-key="(row: TableSelection) => row.name"
              @selection-change="(selection: TableSelection[]) => handleColumnSelectionChange(row.name, selection)"
            >
              <el-table-column
                type="selection"
                width="40"
                :selectable="() => true"
                :reserve-selection="true"
              />
              <el-table-column
                label="列名"
                prop="name"
                min-width="180"
              />
              <el-table-column label="掩码规则" min-width="300">
                <template #default="{ row: column }">
                  <div class="mask-rule-cell" v-if="column.selected">
                    <el-select
                      v-model="column.maskRule.rule_type"
                      placeholder="选择规则"
                      @change="(ruleType: string) => handleMaskRuleChange(row.name, column.name, ruleType)"
                      style="width: 120px"
                      size="small"
                    >
                      <el-option label="无" value="" />
                <el-option label="哈希" value="hash" />
                <el-option label="固定值" value="fixed" />
                <el-option label="模式" value="pattern" />
              </el-select>
              
              <el-input
                      v-if="column.maskRule?.rule_type && column.maskRule.rule_type !== 'hash'"
                      v-model="column.maskRule.pattern"
                      placeholder="请输入替换值或模式"
                      @change="() => updateMaskRules()"
                      style="width: 200px"
                      size="small"
                    />
                    <span v-else-if="column.maskRule?.rule_type === 'hash'" class="text-muted">
                      使用哈希加密
                    </span>
                  </div>
                </template>
              </el-table-column>
            </el-table>
            <div v-else class="loading-placeholder">
              <el-icon class="is-loading"><Loading /></el-icon>
              加载列信息...
            </div>
          </div>
          </template>
        </el-table-column>
        
      <el-table-column type="selection" width="55" :reserve-selection="true" />
      
      <el-table-column label="表名" min-width="200">
          <template #default="{ row }">
          <div class="table-name-cell">
            <span>{{ row.name }}</span>
            <el-tag size="small" type="info" class="table-tag">
              {{ row.name.includes('_') ? row.name.split('_')[0] : '默认' }}
            </el-tag>
            <div class="table-status">
              <el-tag 
                size="small" 
                :type="selectedTableColumns[row.name]?.length ? 'success' : 'info'"
              >
                {{ selectedTableColumns[row.name]?.length || 0 }} 列
              </el-tag>
              <el-tag 
                size="small" 
                :type="getMaskRuleCount(row.name) > 0 ? 'warning' : 'info'"
              >
                {{ getMaskRuleCount(row.name) }} 规则
              </el-tag>
            </div>
          </div>
          </template>
        </el-table-column>
      </el-table>

    <div class="action-bar" v-if="selectedTables.length > 0">
      <el-button
        type="primary"
        :loading="loading"
        @click="$emit('start-task')"
      >
        启动任务
      </el-button>
    </div>
  </el-form-item>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from 'element-plus';
import { Refresh, Loading, Check } from '@element-plus/icons-vue';
import type { Config } from '../types';

const props = defineProps<{
  config: Config;
  loading: boolean;
}>();

const emit = defineEmits<{
  'update:selectedTables': [tables: any[]],
  'start-task': [],
}>();

const tableLoading = ref(false);
const columnLoading = ref<{ [key: string]: boolean }>({});
const availableTables = ref<string[]>([]);
const selectedTables = ref<string[]>([]);
const selectedTableColumns = ref<{ [key: string]: string[] }>({});
const tableColumns = ref<{ [key: string]: string[] }>({});
const maskRules = ref<{ [key: string]: any[] }>({});
const tableOptions = ref<{ [key: string]: { structureOnly: boolean; ignoreForeignKeys: boolean } }>({});

interface TableSelection {
  name: string;
  selected?: boolean;
  maskRule?: {
    column: string;
    rule_type: string;
    pattern?: string;
  };
}

interface TableRow {
  name: string;
}

const loadTables = async () => {
  if (!props.config || !props.config.source_db) {
    ElMessage.warning('请先配置源数据库连接');
    return;
  }
  
  try {
    tableLoading.value = true;
    console.log('开始加载表列表');
    const tables = await invoke<string[]>('get_tables', { config: props.config });
    console.log('获取到表列表:', tables);
    availableTables.value = tables;
    
    // 保持现有的选择状态
    const previousSelection = selectedTables.value;
    selectedTables.value = previousSelection.filter(table => tables.includes(table));
    
    // 恢复已保存的配置
    if (props.config.tables && props.config.tables.length > 0) {
      console.log('恢复已保存的表配置:', props.config.tables);
      props.config.tables.forEach(table => {
        if (typeof table === 'string') {
          if (tables.includes(table)) {
            selectedTables.value.push(table);
          }
        } else if (table.name && tables.includes(table.name)) {
          selectedTables.value.push(table.name);
          if (table.columns) {
            selectedTableColumns.value[table.name] = table.columns;
          }
          if (table.mask_rules) {
            maskRules.value[table.name] = table.mask_rules;
          }
        }
      });
    }
  } catch (error) {
    console.error('加载表失败:', error);
    ElMessage.error('加载表失败: ' + error);
  } finally {
    tableLoading.value = false;
  }
};

const loadTableColumns = async (tableName: string) => {
  if (!props.config || !tableName) return;
  
  try {
    columnLoading.value[tableName] = true;
    console.log('开始加载表列:', tableName);
      const columns = await invoke<string[]>('get_table_columns', {
        config: props.config,
        tableName
    });
    console.log('获取到表列:', columns);
    tableColumns.value[tableName] = columns;
    
    // 保持现有的选择状态
    if (!selectedTableColumns.value[tableName]) {
      selectedTableColumns.value[tableName] = [...columns];
    } else {
      // 过滤掉不存在的列
      selectedTableColumns.value[tableName] = selectedTableColumns.value[tableName]
        .filter(col => columns.includes(col));
    }
    
    if (!maskRules.value[tableName]) {
      maskRules.value[tableName] = [];
    }
  } catch (error) {
    console.error('加载表结构失败:', error);
    ElMessage.error('加载表结构失败: ' + error);
  } finally {
    columnLoading.value[tableName] = false;
  }
};

const refreshTableColumns = async (tableName: string) => {
  await loadTableColumns(tableName);
  ElMessage.success('刷新列成功');
};

const saveTableConfig = () => {
  const selectedTablesConfig = selectedTables.value.map(tableName => ({
    name: tableName,
    columns: selectedTableColumns.value[tableName] || [],
    mask_rules: maskRules.value[tableName] || [],
    structure_only: tableOptions.value[tableName]?.structureOnly || false,
    ignore_foreign_keys: tableOptions.value[tableName]?.ignoreForeignKeys || false
  }));
  emit('update:selectedTables', selectedTablesConfig);
  ElMessage.success('保存配置成功');
};

const handleTableSelectionChange = (selection: TableRow[]) => {
  console.log('表格选择变化:', selection);
  selectedTables.value = selection.map(item => item.name);
  
  // 更新父组件的选中状态
  const selectedTablesConfig = selectedTables.value.map(tableName => ({
    name: tableName,
    columns: selectedTableColumns.value[tableName] || [],
    mask_rules: maskRules.value[tableName] || [],
    structure_only: tableOptions.value[tableName]?.structureOnly || false,
    ignore_foreign_keys: tableOptions.value[tableName]?.ignoreForeignKeys || false
  }));
  emit('update:selectedTables', selectedTablesConfig);
};

const handleColumnSelectionChange = (tableName: string, selection: TableSelection[]) => {
  selectedTableColumns.value[tableName] = selection.map(item => item.name);
};

const handleExpandChange = async (row: TableRow, expanded: boolean) => {
  if (expanded) {
    initTableOptions(row.name);
    if (!tableColumns.value[row.name]) {
      await loadTableColumns(row.name);
    }
  }
};

const handleTableSelect = async (selection: TableRow[], row: TableRow) => {
  if (selection.includes(row)) {
    initTableOptions(row.name);
    if (!tableColumns.value[row.name]) {
      await loadTableColumns(row.name);
    }
  }
  handleTableSelectionChange(selection);
};

const handleMaskRuleChange = (tableName: string, columnName: string, ruleType: string) => {
  const rules = maskRules.value[tableName] || [];
  const existingRule = rules.find(rule => rule.column === columnName);
  
  if (existingRule) {
    existingRule.rule_type = ruleType;
  } else {
    rules.push({
      column: columnName,
      rule_type: ruleType,
      pattern: ''
    });
  }
  
  maskRules.value[tableName] = rules;
  updateTableConfig();
};

const updateMaskRules = () => {
  const selectedTablesConfig = selectedTables.value.map(name => ({
    name,
    columns: selectedTableColumns.value[name] || [],
    mask_rules: maskRules.value[name] || [],
    structure_only: tableOptions.value[name]?.structureOnly || false,
    ignore_foreign_keys: tableOptions.value[name]?.ignoreForeignKeys || false
  }));
  emit('update:selectedTables', selectedTablesConfig);
};

const getMaskRuleCount = (tableName: string): number => {
  return maskRules.value[tableName]?.length || 0;
};

// 初始化表选项
const initTableOptions = (tableName: string) => {
  if (!tableOptions.value[tableName]) {
    tableOptions.value[tableName] = {
      structureOnly: false,
      ignoreForeignKeys: false
    };
  }
};

// 处理表选项变化
const handleTableOptionChange = (tableName: string) => {
  const options = tableOptions.value[tableName];
  if (options.structureOnly) {
    // 如果选择只复制结构，清空列选择和掩码规则
    selectedTableColumns.value[tableName] = [];
    maskRules.value[tableName] = [];
  }
  updateTableConfig();
};

// 更新表配置
const updateTableConfig = () => {
  const selectedTablesConfig = selectedTables.value.map(tableName => ({
    name: tableName,
    columns: selectedTableColumns.value[tableName] || [],
    mask_rules: maskRules.value[tableName] || [],
    structure_only: tableOptions.value[tableName]?.structureOnly || false,
    ignore_foreign_keys: tableOptions.value[tableName]?.ignoreForeignKeys || false
  }));
  emit('update:selectedTables', selectedTablesConfig);
};

// 监听选中表的变化
watch(selectedTables, (newTables) => {
  console.log('选中表发生变化:', newTables);
});

// 监听列选择的变化
watch(selectedTableColumns, (newColumns) => {
  console.log('列选择发生变化:', newColumns);
}, { deep: true });
</script>

<style scoped>
.table-header {
  margin-bottom: 16px;
  display: flex;
  justify-content: flex-end;
}

.table-actions {
  display: flex;
  gap: 8px;
}

.column-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding: 0 12px;
}

.column-header span {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.el-form-item.is-required :deep(.el-table) {
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  max-height: 600px;
  overflow-y: auto;
}

:deep(.el-table__header) {
  background-color: var(--el-fill-color-light);
}

:deep(.el-table__row) {
  cursor: pointer;
}

:deep(.el-table__row:hover) {
  background-color: var(--el-fill-color-lighter) !important;
}

.table-tag {
  margin-left: 8px;
}

.expanded-content {
  padding: 20px;
}

.table-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.table-status {
  margin-left: auto;
  display: flex;
  gap: 8px;
}

.text-muted {
  color: var(--el-text-color-secondary);
  font-style: italic;
  padding: 0 8px;
}

:deep(.el-table--small) {
  font-size: 12px;
}

:deep(.el-table--border) {
  border-radius: 4px;
  overflow: hidden;
}

:deep(.el-table__inner-wrapper) {
  border-radius: 4px;
}

.mask-rule-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

:deep(.el-select .el-input__wrapper),
:deep(.el-input .el-input__wrapper) {
  box-shadow: 0 0 0 1px var(--el-border-color) inset;
  padding: 0 8px;
}

.loading-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px;
  color: var(--el-text-color-secondary);
  background-color: var(--el-bg-color);
  border-radius: 4px;
  border: 1px solid var(--el-border-color);
}

.action-bar {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.table-options {
  margin-bottom: 16px;
  padding: 12px;
  background-color: var(--el-fill-color-light);
  border-radius: 4px;
  display: flex;
  gap: 24px;
}
</style> 