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
          {{ $t('tableConfig.refreshTableList') }}
        </el-button>
      </div>
    </div>

    <el-table
      :data="tableConfig"
      style="width: 100%"
      v-loading="tableLoading"
      row-key="name"
    >
      <el-table-column type="expand">
          <template #default="{ row }">
          <div class="expanded-content">
            <div class="table-options">
              <el-checkbox 
                v-model="tableConfig![row.name].structure_only"
              >
                {{ $t('tableConfig.structureOnly') }}
              </el-checkbox>
              <el-checkbox 
                v-model="tableConfig![row.name].ignore_foreign_keys"
              >
              >
                {{ $t('tableConfig.ignoreForeignKeys') }}
              </el-checkbox>
              <template v-if="tableStructureChanges.has(row.name)">
                <el-tag 
                  v-if="tableStructureChanges.get(row.name) === TableStructureChangeType.Added"
                  type="info"
                >
                  {{ $t('tableConfig.added') }}
                </el-tag>
                <el-tag 
                  v-else
                  type="danger"
                >
                  {{ tableStructureChanges.get(row.name) === TableStructureChangeType.Removed ? $t('tableConfig.removed') :  $t('tableConfig.updated') }}
                </el-tag>
              </template>
            </div>
            
            <div class="column-header">
              <span>{{ $t('tableConfig.columns.columnName') }}</span>
              <div>
                <el-button 
                  type="primary" 
                  size="small" 
                  link
                  @click="refreshTableColumns(row.name)"
                  :loading="columnLoading[row.name]"
                >
                  <el-icon><Refresh /></el-icon>
                  {{ $t('tableConfig.refreshColumns') }}
                </el-button>
              </div>
            </div>
            
            <el-table
              v-if="tableConfig![row.name].columns"
              :data="tableConfig![row.name].columns"
              style="width: 100%"
              size="small"
              border
              row-key="name"
            >
              <el-table-column
                type="selection"
                width="40"
                :selectable="() => true"
                :reserve-selection="true"
              />
              <el-table-column
                :label="$t('tableConfig.columns.columnName')"
                prop="name"
                min-width="180"
              >
                <template #default="{ row: column }">
                  <div class="column-name-cell">
                    {{ column.name }}
                    <el-tag v-if="column.isNew" type="success" size="small" effect="plain">{{ $t('tableConfig.new') }}</el-tag>
                  </div>
                </template>
              </el-table-column>
              <el-table-column :label="$t('tableConfig.columns.maskRule')" min-width="300">
                <template #default="{ row: column }">
                  <div class="mask-rule-cell" v-if="column.selected">
                    <el-select
                      v-model="column.maskRule.rule_type"
                      :placeholder="$t('tableConfig.selectRule')"
                      style="width: 120px"
                      size="small"
                    >
                      <el-option :label="$t('tableConfig.rules.none')" value="" />
                      <el-option :label="$t('tableConfig.rules.hash')" value="hash" />
                      <el-option :label="$t('tableConfig.rules.fixed')" value="fixed" />
                      <el-option :label="$t('tableConfig.rules.pattern')" value="pattern" />
                    </el-select>
                    
                    <el-input
                      v-if="column.maskRule?.rule_type && column.maskRule.rule_type !== 'hash'"
                      v-model="column.maskRule.pattern"
                      :placeholder="$t('tableConfig.enterReplacement')"
                      style="width: 200px"
                      size="small"
                    />
                    <span v-else-if="column.maskRule?.rule_type === 'hash'" class="text-muted">
                      {{ $t('tableConfig.useHash') }}
                    </span>
                  </div>
                </template>
              </el-table-column>
            </el-table>
            <div v-else class="loading-placeholder">
              <el-icon class="is-loading"><Loading /></el-icon>
              {{ $t('tableConfig.loadingColumnInfo') }}...
            </div>
          </div>
          </template>
        </el-table-column>
        
      <el-table-column type="selection" width="55" :reserve-selection="true" />
      
      <el-table-column :label="$t('tableConfig.columns.tableName')" min-width="200">
          <template #default="{ row }">
          <div class="table-name-cell">
            <span>{{ row.name }}</span>
            <el-tag size="small" type="info" class="table-tag">
              {{ row.name.includes('_') ? row.name.split('_')[0] : $t('tableConfig.default') }}
            </el-tag>
            <div class="table-status">
              <el-tag 
                size="small" 
                :type="tableConfig![row.name].columns?.length ? 'success' : 'info'"
              >
                {{ $t('tableConfig.columnCount', { count: tableConfig![row.name].columns?.length || 0 }) }}
              </el-tag>
              <el-tag 
                size="small" 
                :type="tableConfig![row.name].columns?.length ? 'warning' : 'info'"
              >
                {{ $t('tableConfig.ruleCount', { count: tableConfig![row.name].columns?.length || 0 }) }}
              </el-tag>
              <el-tag
                v-if="tableStructureChanges.has(row.name)"
                size="small"
                type="warning"
              >
                {{ $t('tableConfig.structureChanged') }}
              </el-tag>
            </div>
          </div>
          </template>
        </el-table-column>
      </el-table>
  </el-form-item>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Refresh, Loading } from '@element-plus/icons-vue';
import { databaseApi } from '@/services/api';
import { DatabaseConfig, TableConfig } from '@/types';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
  sourceDb: DatabaseConfig;
}>();

const tableConfig = defineModel<TableConfig[]>()

const tableLoading = ref(false);
const columnLoading = ref<{ [key: string]: boolean }>({});

enum TableStructureChangeType {
  Added = 'added',
  Removed = 'removed',
  Updated = 'updated',
}
// 表结构变更记录
const tableStructureChanges = ref<Map<string, TableStructureChangeType>>(new Map());


const loadTables = async () => {
  if (!props.sourceDb) {
    ElMessage.warning('请先配置源数据库连接');
    return;
  }
  
  try {
    tableLoading.value = true;
    console.log('开始加载表列表');
    const tables = await databaseApi.getTables(props.sourceDb);
    console.log('获取到表列表:', tables);
    
    // 恢复自动加载所有选中表的列
    for (const tableName of tables) {
      await loadTableColumns(tableName);
    }
  } catch (error) {
    console.error('加载表失败:', error);
    ElMessage.error('加载表失败: ' + error);
  } finally {
    tableLoading.value = false;
  }
};

const loadTableColumns = async (tableName: string) => {
  if (!props.sourceDb || !tableName) return;
  
  try {
    columnLoading.value[tableName] = true;
    console.log('开始加载表列:', tableName);
    const columns = await databaseApi.getTableColumns(props.sourceDb, tableName);
    console.log('获取到表列:', columns);
    
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


onMounted(() => {
  if (props.sourceDb) {
    loadTables();
  }
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