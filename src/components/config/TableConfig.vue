<template>
  <el-form-item :label="$t('tableConfig.treeTitle')" required>
    <div class="table-config-container">

      <div class="table-header">
        <div class="table-actions">
          <el-button-group>
            <el-button 
              size="small"
            >
              <el-icon><ArrowDown /></el-icon>
              {{ $t('tableConfig.expandAll') }}
            </el-button>
            <el-button 
              size="small"
            >
              <el-icon><ArrowUp /></el-icon>
              {{ $t('tableConfig.collapseAll') }}
            </el-button>
          </el-button-group>
  
          <el-button 
            type="primary" 
            size="small" 
            @click="mergeTableConfig"
            :loading="tableLoading"
          >
            <el-icon><Refresh /></el-icon>
            {{ $t('tableConfig.refreshTableList') }}
          </el-button>
          
          <el-input
            v-model="searchText"
            :placeholder="$t('tableConfig.searchPlaceholder')"
            size="small"
            clearable
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
      </div>
      
      <el-tree
        :data="tableConfig"
        :props="{
          label: 'name',
          children: 'columns',
        }"
        show-checkbox
        node-key="name"
      >
      </el-tree>
    </div>
  </el-form-item>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { ElMessage, ElTree } from 'element-plus';
import { Refresh, Search, ArrowDown, ArrowUp } from '@element-plus/icons-vue';
import { configApi, databaseApi } from '@/services/api';
import { DatabaseConfig, TableConfig, ColumnConfig, MaskRule } from '@/types';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
  sourceDb: DatabaseConfig;
}>();

const tableConfig = defineModel<TableConfig[]>({ required: true });
const tableLoading = ref(false);
const searchText = ref('');

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


</style> 