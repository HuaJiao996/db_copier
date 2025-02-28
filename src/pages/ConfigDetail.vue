<template>
  <div class="config-detail">
    <div class="header">
      <h2>{{ isCreating ? t('configDetail.title.new') : t('configDetail.title.edit', { name: currentConfig.name }) }}</h2>
      <el-button-group>
        <el-button @click="goBack">
          {{ t('common.cancel') }}
        </el-button>
        <el-button type="primary" @click="saveConfig" :loading="loading">
          {{ t('common.save') }}
        </el-button>
        <el-button 
          type="success" 
          @click="startTask" 
          :loading="loading" 
          :disabled="!canStartTask"
        >
          {{ t('configManager.startTask') }}
        </el-button>
      </el-button-group>
    </div>

    <el-form
      ref="formRef"
      :model="currentConfig"
      :rules="rules"
      label-width="120px"
      class="config-form"
      v-loading="loading"
    >
      <el-form-item :label="t('configDetail.configName')" prop="name">
        <el-input v-model="currentConfig.name" :disabled="!isCreating" />
      </el-form-item>

      <el-tabs v-model="activeTab" class="config-tabs">
        <el-tab-pane lazy :label="t('configDetail.tabs.connection')" name="connection">
          <div class="database-section">
            <!-- 源数据库配置 -->
            <DatabaseConfig
              type="source"
              v-model="currentConfig.source_db"
            />

            <!-- 目标数据库配置 -->
            <DatabaseConfig
              type="target"
              v-model="currentConfig.target_db"
            />
          </div>
        </el-tab-pane>

        <el-tab-pane :label="t('configDetail.tabs.tables')" name="tables" lazy>
          <!-- 表配置 -->
          <TableConfig
            v-model="currentConfig.tables"
            :source-db="currentConfig.source_db"
          />
        </el-tab-pane>
      </el-tabs>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from 'vue';
import type { FormInstance, FormRules } from 'element-plus';
import DatabaseConfig from '@/components/config/DatabaseConfig.vue';
import TableConfig from '@/components/config/TableConfig.vue';
import { useRouter } from 'vue-router';
import type { Config } from '@/types';
import { configApi, taskApi } from '@/services/api';
import { useLoading } from '@/composables/useLoading';
import { useNotification } from '@/composables/useNotification';
import { formatError } from '@/utils/error';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  isCreating: boolean;
  configName?: string;
}>();

const router = useRouter();
const formRef = ref<FormInstance>();
const activeTab = ref('connection');
const { isLoading: loading, runWithLoading } = useLoading();
const { showSuccess, showError, showWarning } = useNotification();
const { t } = useI18n();

const initConfig = (): Config => ({
  name: props.configName || '',
  source_db: {
    host: '',
    port: 5432,
    database: '',
    username: '',
    password: '',
    ssl_mode: 'prefer'
  },
  target_db: {
    host: '',
    port: 5432,
    database: '',
    username: '',
    password: '',
    ssl_mode: 'prefer'
  },
  tables: []
});

const currentConfig = ref<Config>(initConfig());

const rules = {
  name: [
    { required: true, message: t('configDetail.rules.nameRequired'), trigger: 'blur' },
    { min: 2, max: 50, message: t('configDetail.rules.nameLength'), trigger: 'blur' }
  ],
} as FormRules;

const canStartTask = computed(() => {
  return currentConfig.value.tables && currentConfig.value.tables.length > 0;
});


// 监听activeTab变化，当切换到表和列配置时自动加载表结构
watch(activeTab, async (newTab) => {
  if (newTab === 'tables' && currentConfig.value.source_db) {
    // 确保数据库连接信息已填写
    if (!currentConfig.value.source_db.host || 
        !currentConfig.value.source_db.database || 
        !currentConfig.value.source_db.username) {
      showWarning(t('configDetail.messages.completeDbConfig'));
      activeTab.value = 'connection';
      return;
    }
  }
});

const loadConfig = async () => {
  if (!props.configName) return;
  
  const configName = props.configName; // 确保非空
  
  await runWithLoading(async () => {
    try {
      const config = await configApi.load(configName);

      // 更新当前配置
      currentConfig.value = config;
    } catch (error) {
      showError(t('configDetail.errors.loadFailed', { error: formatError(error) }));
      router.push('/config');
    }
  });
};

const saveConfig = async () => {
  if (!formRef.value) return;

  try {
    await formRef.value.validate();
    
    await runWithLoading(async () => {
      // 强制等待一个tick，确保所有数据更新都已完成
      await nextTick();
      
      await configApi.save(currentConfig.value);
      showSuccess(t('configDetail.messages.saveSuccess'));
    });
  } catch (error) {
    showError(t('configDetail.errors.saveFailed', { error: formatError(error) }));
  }
};

const startTask = async () => {
  if (!canStartTask.value) {
    showWarning(t('configDetail.messages.selectTables'));
    return;
  }

  await runWithLoading(async () => {
    try {
      await taskApi.start(currentConfig.value);
      showSuccess(t('configDetail.messages.taskCreated'));
      router.push('/');
    } catch (error) {
      showError(t('configDetail.errors.taskCreateFailed', { error: formatError(error) }));
    }
  });
};

const goBack = () => {
  router.push('/config');
};

onMounted(() => {
  if (!props.isCreating && props.configName) {
    loadConfig();
  } else {
    currentConfig.value = initConfig();
  }
});
</script>

<style scoped>
.config-detail {
  background-color: var(--el-bg-color);
  border-radius: 8px;
  padding: 20px;
  box-shadow: var(--el-box-shadow-light);
  margin: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
}

.config-form {
  margin-top: 20px;
}

.config-tabs {
  margin-top: 20px;
}

.database-section {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

:deep(.el-tabs__nav) {
  margin-bottom: 20px;
}

:deep(.el-tab-pane) {
  padding: 20px 0;
}

:deep(.el-divider__text) {
  background-color: var(--el-bg-color);
}
</style> 