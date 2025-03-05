<template>
  <el-form-item :label="$t('tableConfig.treeTitle')" required>
    <div class="table-config-container">
      <div class="table-header">
        <div class="table-actions">
          <el-button 
            type="primary" 
            size="small" 
            @click="mergeTableConfig"
            :loading="tableLoading"
          >
            <el-icon><Refresh /></el-icon>
            {{ $t('tableConfig.refreshTableList') }}
          </el-button>
        </div>
      </div>
      
      <el-tree
        ref="treeRef"
        :data="tableConfig"
        :props="{
          label: 'name',
          children: 'columns',
        }"
        node-key="name"
      >
        <template #default="{ node, data }">
          <div class="custom-tree-node">
            <div class="node-content">
              <el-checkbox v-model="data.selected" @click.stop></el-checkbox>
              <span class="node-label">{{ node.label }}</span>
              <!-- 表级别的配置选项 -->
              <template v-if="data.columns">
                <div class="table-options">
                  <el-checkbox 
                    v-model="data.structure_only"
                    @click.stop
                  >
                    {{ $t('tableConfig.structureOnly') }}
                  </el-checkbox>
                  <el-checkbox 
                    v-model="data.ignore_foreign_keys"
                    @click.stop
                  >
                    {{ $t('tableConfig.ignoreForeignKeys') }}
                  </el-checkbox>
                </div>
              </template>

              <!-- 列级别的配置选项 -->
              <template v-else>
                <div class="column-options">
                  <template v-if="data.selected">
                    <el-select
                      v-model="data.mask_rule?.rule_type"
                      size="small"
                      @click.stop
                    >
                      <el-option :label="$t('tableConfig.rules.none')" value="" />
                      <el-option :label="$t('tableConfig.rules.hash')" value="hash" />
                      <el-option :label="$t('tableConfig.rules.fixed')" value="fixed" />
                      <el-option :label="$t('tableConfig.rules.pattern')" value="pattern" />
                    </el-select>

                    <el-input
                      v-if="data.mask_rule?.rule_type && data.mask_rule.rule_type !== 'hash'"
                      v-model="data.mask_rule?.pattern"
                      :placeholder="$t('tableConfig.enterReplacement')"
                      size="small"
                      @click.stop
                    />
                    <span v-else-if="data.mask_rule?.rule_type === 'hash'" class="text-muted">
                      {{ $t('tableConfig.useHash') }}
                    </span>
                  </template>
                </div>
              </template>
            </div>
          </div>
        </template>
      </el-tree>
    </div>
  </el-form-item>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Refresh } from '@element-plus/icons-vue';
import { configApi } from '@/services/api';
import { DatabaseConfig, TableConfig } from '@/types';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
  sourceDb: DatabaseConfig;
}>();

const tableConfig = defineModel<TableConfig[]>({ required: true });
const tableLoading = ref(false);

const mergeTableConfig = async () => {
  if (!props.sourceDb) {
    ElMessage.warning(t('configDetail.messages.completeDbConfig'));
    return;
  }
  
  try {
    tableLoading.value = true;
    console.log('开始加载表列表');
    const mergedTableConfig = await configApi.mergeTableConfig(props.sourceDb, tableConfig.value);
    tableConfig.value = mergedTableConfig;
    // 重置表信息变更状态
    ElMessage.success(t('tableConfig.messages.loadSuccess'));
  } catch (error) {
    console.error('加载表失败:', error);
    ElMessage.error(t('tableConfig.messages.loadFailed') + ': ' + error);
  } finally {
    tableLoading.value = false;
  }
};

onMounted(() => {
  if (props.sourceDb) {
    mergeTableConfig();
  }
});
</script>

<style scoped>
.table-config-container {
  width: 100%;
}

.custom-tree-node {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 14px;
  padding: 0 8px;
  width: 100%;
}

.node-content {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.node-label {
  min-width: 120px;
}

.table-options,
.column-options {
  display: flex;
  gap: 16px;
  align-items: center;
}

:deep(.el-tree-node__content) {
  height: auto;
  min-height: 32px;
  padding: 4px 0;
}

:deep(.el-select) {
  width: 120px;
}

:deep(.el-checkbox) {
  margin-right: 0;
  white-space: nowrap;
}

:deep(.el-tree-node.is-checked > .el-tree-node__content) {
  background-color: var(--el-color-primary-light-9);
}
</style> 