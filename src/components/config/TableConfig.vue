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
      :data="availableTables.map(name => ({ 
        name,
        hasChanges: tableStructureChanges[name]?.hasChanges || false
      }))"
      style="width: 100%"
      v-loading="tableLoading"
      row-key="name"
      @selection-change="handleTableSelectionChange"
      @expand-change="handleExpandChange"
    >
      <el-table-column type="expand">
          <template #default="{ row }">
          <div class="expanded-content">
            <div class="table-options">
              <el-checkbox 
                v-model="localTableOptions[row.name].structureOnly"
                @change="() => updateTableOption(row.name, 'structureOnly', localTableOptions[row.name].structureOnly)"
              >
                仅复制表结构
              </el-checkbox>
              <el-checkbox 
                v-model="localTableOptions[row.name].ignoreForeignKeys"
                @change="() => updateTableOption(row.name, 'ignoreForeignKeys', localTableOptions[row.name].ignoreForeignKeys)"
              >
                忽略外键关联
              </el-checkbox>
            </div>
            
            <!-- 表结构变更提示 -->
            <div v-if="row.hasChanges" class="structure-changes-alert">
              <el-alert
                title="表结构已变更"
                type="warning"
                :closable="false"
                show-icon
              >
                <template #default>
                  <div class="structure-changes">
                    <div v-if="tableStructureChanges[row.name].added.length > 0">
                      <strong>新增列:</strong> 
                      <el-tag 
                        v-for="col in tableStructureChanges[row.name].added" 
                        :key="col" 
                        type="success" 
                        size="small"
                        class="structure-change-tag"
                      >
                        {{ col }}
                      </el-tag>
                    </div>
                    <div v-if="tableStructureChanges[row.name].removed.length > 0">
                      <strong>移除列:</strong> 
                      <el-tag 
                        v-for="col in tableStructureChanges[row.name].removed" 
                        :key="col" 
                        type="danger" 
                        size="small"
                        class="structure-change-tag"
                      >
                        {{ col }}
                      </el-tag>
                    </div>
                    <div class="structure-changes-actions">
                      <el-button 
                        type="primary" 
                        size="small" 
                        @click="updateTableStructure(row.name)"
                      >
                        更新表结构
                      </el-button>
                    </div>
                  </div>
                </template>
              </el-alert>
            </div>
            
            <div class="column-header">
              <span>列配置</span>
              <div>
                <span v-if="tableLastUpdated[row.name]" class="last-updated">
                  上次更新: {{ formatDate(tableLastUpdated[row.name]) }}
                </span>
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
            </div>
            
            <el-table
              v-if="tableColumns[row.name]"
              :data="tableColumns[row.name].map(col => ({
                name: col,
                selected: selectedTableColumns[row.name]?.includes(col),
                maskRule: maskRules[row.name]?.find(rule => rule.column === col) || { column: col, rule_type: '', pattern: '' },
                isNew: tableStructureChanges[row.name]?.added.includes(col)
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
              >
                <template #default="{ row: column }">
                  <div class="column-name-cell">
                    {{ column.name }}
                    <el-tag v-if="column.isNew" type="success" size="small" effect="plain">新</el-tag>
                  </div>
                </template>
              </el-table-column>
              <el-table-column label="掩码规则" min-width="300">
                <template #default="{ row: column }">
                  <div class="mask-rule-cell" v-if="column.selected">
                    <el-select
                      v-model="column.maskRule.rule_type"
                      placeholder="选择规则"
                      style="width: 120px"
                      size="small"
                      @change="(val) => handleMaskRuleChange(row.name, column.name, val)"
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
                      style="width: 200px"
                      size="small"
                      @input="(val) => handleMaskRulePatternChange(row.name, column.name, val)"
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
              <el-tag
                v-if="row.hasChanges"
                size="small"
                type="warning"
              >
                结构已变更
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
import { ref, watch, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh, Check, Loading } from '@element-plus/icons-vue';
import { databaseApi } from '@/services/api';
import { Config } from '@/types';

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
const selectedTables = defineModel<any[]>('selectedTables', {
  default: [] // 提供默认值，避免undefined
});
const selectedTableColumns = ref<{ [key: string]: string[] }>({});
const tableColumns = ref<{ [key: string]: string[] }>({});
const maskRules = ref<{ [key: string]: any[] }>({});
const tableOptions = ref<{ [key: string]: TableOptions }>({});

// 表结构变更记录
const tableStructureChanges = ref<Record<string, { added: string[], removed: string[], hasChanges: boolean }>>({});
// 表结构最后更新时间
const tableLastUpdated = ref<Record<string, string>>({});

// 本地表选项，用于v-model绑定
const localTableOptions = ref<{ [key: string]: { structureOnly: boolean; ignoreForeignKeys: boolean } }>({});

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

// 定义TableOptions类型
interface TableOptions {
  structureOnly: boolean;
  ignoreForeignKeys: boolean;
  columns: Array<{
    name: string;
    selected: boolean;
    maskRule?: {
      rule_type: string;
      pattern: string;
    };
  }>;
}

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr);
  return date.toLocaleString();
};

const loadTables = async () => {
  if (!props.config || !props.config.source_db) {
    ElMessage.warning('请先配置源数据库连接');
    return;
  }
  
  try {
    tableLoading.value = true;
    console.log('开始加载表列表');
    const tables = await databaseApi.getTables(props.config);
    console.log('获取到表列表:', tables);
    availableTables.value = tables;
    
    // 保持现有的选择状态
    const previousSelection = selectedTables.value;
    selectedTables.value = previousSelection.filter(table => tables.includes(table));
    
    // 恢复已保存的配置
    if (props.config.tables && props.config.tables.length > 0) {
      console.log('恢复已保存的表配置:', props.config.tables);
      props.config.tables.forEach((table: any) => {
        if (typeof table === 'string') {
          if (tables.includes(table)) {
            selectedTables.value.push(table);
          }
        } else if (table.name && tables.includes(table.name)) {
          // 确保表在选中列表中
          if (!selectedTables.value.includes(table.name)) {
            selectedTables.value.push(table.name);
          }
          
          // 恢复列配置
          if (table.columns) {
            selectedTableColumns.value[table.name] = table.columns;
          }
          
          // 恢复掩码规则
          if (table.mask_rules) {
            maskRules.value[table.name] = table.mask_rules;
          }
          
          // 恢复表选项
          if (table.structure_only !== undefined || table.ignore_foreign_keys !== undefined) {
            tableOptions.value[table.name] = {
              structureOnly: table.structure_only || false,
              ignoreForeignKeys: table.ignore_foreign_keys || false,
              columns: [] // 添加空的columns数组
            };
          } else {
            // 初始化默认选项
            initTableOptions(table.name);
          }
          
          // 保存表结构最后更新时间
          if (table.last_updated) {
            tableLastUpdated.value[table.name] = table.last_updated;
          } else {
            // 如果没有最后更新时间，设置为当前时间
            tableLastUpdated.value[table.name] = new Date().toISOString();
          }
        }
      });
    }
    
    // 恢复自动加载所有选中表的列
    for (const tableName of selectedTables.value) {
      await loadTableColumns(tableName, false);
    }
  } catch (error) {
    console.error('加载表失败:', error);
    ElMessage.error('加载表失败: ' + error);
  } finally {
    tableLoading.value = false;
  }
};

const loadTableColumns = async (tableName: string, forceRefresh = false) => {
  if (!props.config || !tableName) return;
  
  try {
    columnLoading.value[tableName] = true;
    console.log('开始加载表列:', tableName);
    const columns = await databaseApi.getTableColumns(props.config, tableName);
    console.log('获取到表列:', columns);
    
    // 检查表结构变更
    if (selectedTableColumns.value[tableName] && !forceRefresh) {
      const savedColumns = selectedTableColumns.value[tableName];
      tableStructureChanges.value[tableName] = databaseApi.compareTableStructure(columns, savedColumns);
    } else {
      tableStructureChanges.value[tableName] = { added: [], removed: [], hasChanges: false };
    }
    
    tableColumns.value[tableName] = columns;
    
    // 如果是首次加载或强制刷新，更新最后更新时间
    if (forceRefresh || !tableLastUpdated.value[tableName]) {
      tableLastUpdated.value[tableName] = new Date().toISOString();
    }
    
    // 保持现有的选择状态
    if (!selectedTableColumns.value[tableName]) {
      selectedTableColumns.value[tableName] = [...columns];
    } else if (forceRefresh) {
      // 如果是强制刷新，则更新选中列
      // 保留原有选中的列，并添加新列
      const currentSelected = selectedTableColumns.value[tableName];
      const newColumns = columns.filter(col => !tableColumns.value[tableName].includes(col));
      selectedTableColumns.value[tableName] = [
        ...currentSelected.filter(col => columns.includes(col)),
        ...newColumns
      ];
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

// 更新表结构
const updateTableStructure = (tableName: string) => {
  if (!tableColumns.value[tableName]) return;
  
  // 更新选中的列
  const currentColumns = tableColumns.value[tableName];
  const currentSelected = selectedTableColumns.value[tableName] || [];
  
  // 添加新列到选中列表
  const newColumns = tableStructureChanges.value[tableName].added;
  const updatedColumns = [
    ...currentSelected.filter(col => currentColumns.includes(col)),
    ...newColumns
  ];
  
  selectedTableColumns.value[tableName] = updatedColumns;
  
  // 更新最后更新时间
  tableLastUpdated.value[tableName] = new Date().toISOString();
  
  // 清除变更记录
  tableStructureChanges.value[tableName] = { added: [], removed: [], hasChanges: false };
  
  // 保存配置
  saveTableConfig();
  
  ElMessage.success(`表 ${tableName} 结构已更新`);
};

const refreshTableColumns = async (tableName: string) => {
  await loadTableColumns(tableName, true);
  ElMessage.success('刷新列成功');
};

const saveTableConfig = () => {
  const selectedTablesConfig = selectedTables.value.map(tableName => ({
    name: tableName,
    columns: selectedTableColumns.value[tableName] || [],
    mask_rules: maskRules.value[tableName] || [],
    structure_only: tableOptions.value[tableName]?.structureOnly || false,
    ignore_foreign_keys: tableOptions.value[tableName]?.ignoreForeignKeys || false,
    last_updated: tableLastUpdated.value[tableName] || new Date().toISOString()
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
    ignore_foreign_keys: tableOptions.value[tableName]?.ignoreForeignKeys || false,
    last_updated: tableLastUpdated.value[tableName] || new Date().toISOString()
  }));
  emit('update:selectedTables', selectedTablesConfig);
};

const handleColumnSelectionChange = (tableName: string, selection: TableSelection[]) => {
  selectedTableColumns.value[tableName] = selection.map(item => item.name);
};

const handleExpandChange = async (row: TableRow, expanded: boolean) => {
  if (expanded) {
    // 确保表选项已初始化
    initTableOptions(row.name);
    
    // 加载表列
    if (!tableColumns.value[row.name]) {
      await loadTableColumns(row.name);
    }
  }
};

const handleTableSelect = async (selection: TableRow[], row: TableRow) => {
  if (selection.includes(row)) {
    // 确保表选项已初始化
    initTableOptions(row.name);
    
    if (!tableColumns.value[row.name]) {
      await loadTableColumns(row.name);
    }
  }
  handleTableSelectionChange(selection);
};

const handleMaskRuleChange = (tableName: string, columnName: string, ruleType: string) => {
  const table = tableOptions.value[tableName];
  if (!table) return;
  
  const column = table.columns.find(col => col.name === columnName);
  if (!column) return;
  
  if (!column.maskRule) {
    column.maskRule = { rule_type: '', pattern: '' };
  }
  
  column.maskRule.rule_type = ruleType;
  
  // 如果规则类型为空，清空模式
  if (!ruleType) {
    column.maskRule.pattern = '';
  }
  
  // 更新掩码规则
  updateMaskRules(tableName);
};

const handleMaskRulePatternChange = (tableName: string, columnName: string, pattern: string) => {
  const table = tableOptions.value[tableName];
  if (!table) return;
  
  const column = table.columns.find(col => col.name === columnName);
  if (!column || !column.maskRule) return;
  
  column.maskRule.pattern = pattern;
  
  // 更新掩码规则
  updateMaskRules(tableName);
};

const updateMaskRules = (tableName: string) => {
  const table = tableOptions.value[tableName];
  if (!table) return;
  
  const rules: any[] = [];
  
  // 从表格列中提取掩码规则
  if (tableColumns.value[tableName]) {
    tableColumns.value[tableName].forEach(columnName => {
      const column = table.columns.find(col => col.name === columnName);
      if (column && column.maskRule && column.maskRule.rule_type) {
        rules.push({
          column: columnName,
          rule_type: column.maskRule.rule_type,
          pattern: column.maskRule.pattern || ''
        });
      }
    });
  }
  
  maskRules.value[tableName] = rules;
  
  // 更新表配置
  updateTableConfig();
};

const getMaskRuleCount = (tableName: string): number => {
  return maskRules.value[tableName]?.length || 0;
};

// 初始化表选项
const initTableOptions = (tableName: string) => {
  // 不再需要检查undefined，因为我们已经提供了默认值
  
  if (!tableOptions.value[tableName]) {
    tableOptions.value[tableName] = {
      structureOnly: false,
      ignoreForeignKeys: false,
      columns: [] // 添加空的columns数组
    };
  }
  
  // 确保columns属性存在
  if (!tableOptions.value[tableName].columns) {
    tableOptions.value[tableName].columns = [];
  }
  
  // 如果表的列已加载，初始化列选项
  if (tableColumns.value[tableName]) {
    // 确保每个列都有对应的选项
    const existingColumns = tableOptions.value[tableName].columns.map(c => c.name);
    
    tableColumns.value[tableName].forEach(columnName => {
      if (!existingColumns.includes(columnName)) {
        tableOptions.value[tableName].columns.push({
          name: columnName,
          selected: true // 默认选中所有列
        });
      }
    });
  }
};

// 更新表选项
const updateTableOption = (tableName: string, option: string, value: boolean) => {
  console.log(`更新表 ${tableName} 选项 ${option}:`, value);
  
  if (!tableOptions.value[tableName]) {
    tableOptions.value[tableName] = {
      structureOnly: false,
      ignoreForeignKeys: false,
      columns: [] // 添加空的columns数组
    };
  }
  
  if (option === 'structureOnly' || option === 'ignoreForeignKeys') {
    tableOptions.value[tableName][option] = value;
  }
  
  // 如果选择只复制结构，清空列选择和掩码规则
  if (option === 'structureOnly' && value) {
    selectedTableColumns.value[tableName] = [];
    maskRules.value[tableName] = [];
  }
  
  updateTableConfig();
};

// 更新表配置
const updateTableConfig = () => {
  console.log('更新表配置');
  
  // 确保所有选中的表都有对应的配置
  const selectedTablesConfig = selectedTables.value.map(tableName => {
    // 如果是字符串，转换为对象
    if (typeof tableName === 'string') {
      return {
        name: tableName,
        columns: selectedTableColumns.value[tableName] || [],
        mask_rules: maskRules.value[tableName] || [],
        structure_only: tableOptions.value[tableName]?.structureOnly || false,
        ignore_foreign_keys: tableOptions.value[tableName]?.ignoreForeignKeys || false,
        last_updated: tableLastUpdated.value[tableName] || new Date().toISOString()
      };
    }
    
    // 如果已经是对象，更新其属性
    if (tableName && typeof tableName === 'object' && 'name' in tableName) {
      const name = tableName.name;
      return {
        ...tableName,
        columns: selectedTableColumns.value[name] || tableName.columns || [],
        mask_rules: maskRules.value[name] || tableName.mask_rules || [],
        structure_only: tableOptions.value[name]?.structureOnly || tableName.structure_only || false,
        ignore_foreign_keys: tableOptions.value[name]?.ignoreForeignKeys || tableName.ignore_foreign_keys || false,
        last_updated: tableLastUpdated.value[name] || tableName.last_updated || new Date().toISOString()
      };
    }
    
    return tableName;
  });
  
  // 更新父组件的选中状态
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

// 暴露方法给父组件
defineExpose({
  loadTables
});

onMounted(() => {
  // 初始化表选项
  availableTables.value.forEach(tableName => {
    if (!tableOptions.value[tableName]) {
      tableOptions.value[tableName] = {
        structureOnly: false,
        ignoreForeignKeys: false,
        columns: [] // 添加空的columns数组
      };
    }
  });
  
  // 不再自动加载表列表，用户需要手动点击刷新按钮
  // if (props.config && props.config.source_db) {
  //   loadTables();
  // }
});
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

.structure-changes-alert {
  margin-bottom: 16px;
}

.structure-changes {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.structure-changes-actions {
  margin-top: 8px;
}

.structure-change-tag {
  margin-right: 4px;
  margin-left: 4px;
}

.column-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.last-updated {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-right: 8px;
}
</style> 