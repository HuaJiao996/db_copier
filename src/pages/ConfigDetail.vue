<template>
  <div class="config-detail">
    <div class="header">
      <h2>{{ isCreating ? '新建配置' : `编辑配置: ${currentConfig.name}` }}</h2>
      <el-button-group>
        <el-button @click="goBack">
          返回
        </el-button>
        <el-button type="primary" @click="saveConfig" :loading="loading">
          保存配置
        </el-button>
        <el-button 
          type="success" 
          @click="startTask" 
          :loading="loading" 
          :disabled="!canStartTask"
        >
          启动任务
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
      <el-form-item label="配置名称" prop="name">
        <el-input v-model="currentConfig.name" :disabled="!isCreating" />
      </el-form-item>

      <el-tabs v-model="activeTab" class="config-tabs">
        <el-tab-pane label="数据库连接" name="connection">
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

        <el-tab-pane label="表和列配置" name="tables">
          <!-- 表配置 -->
          <TableConfig
            ref="tableConfigRef"
            :config="currentConfig"
            :loading="loading"
            v-model:selectedTables="currentConfig.tables"
            @start-task="startTask"
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
import { useLoading } from '@/hooks/useLoading';
import { useNotification } from '@/hooks/useNotification';
import { formatError } from '@/utils/error';

const props = defineProps<{
  isCreating: boolean;
  configName?: string;
}>();

const router = useRouter();
const formRef = ref<FormInstance>();
const tableConfigRef = ref<InstanceType<typeof TableConfig>>();
const activeTab = ref('connection');
const { isLoading: loading, runWithLoading } = useLoading();
const { showSuccess, showError, showWarning } = useNotification();

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
    { required: true, message: '请输入配置名称', trigger: 'blur' },
    { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' }
  ],
} as FormRules;

const canStartTask = computed(() => {
  return currentConfig.value.tables && currentConfig.value.tables.length > 0;
});

// 监听currentConfig变化
watch(currentConfig, (newValue) => {
  console.log('当前配置变化:', JSON.parse(JSON.stringify(newValue)));
}, { deep: true });

// 监听activeTab变化，当切换到表和列配置时自动加载表结构
watch(activeTab, async (newTab) => {
  if (newTab === 'tables' && currentConfig.value.source_db) {
    // 确保数据库连接信息已填写
    if (!currentConfig.value.source_db.host || 
        !currentConfig.value.source_db.database || 
        !currentConfig.value.source_db.username) {
      showWarning('请先完成数据库连接配置');
      activeTab.value = 'connection';
      return;
    }
    // 自动加载表结构
    if (tableConfigRef.value) {
      tableConfigRef.value.loadTables();
    }
  }
});

const loadConfig = async () => {
  if (!props.configName) return;
  
  const configName = props.configName; // 确保非空
  
  await runWithLoading(async () => {
    try {
      const config = await configApi.load(configName);
      console.log('加载到的原始配置:', JSON.stringify(config));
      
      // 确保数据库配置中的ssl_mode有默认值
      if (!config.source_db.ssl_mode) {
        config.source_db.ssl_mode = 'prefer';
      }
      if (!config.target_db.ssl_mode) {
        config.target_db.ssl_mode = 'prefer';
      }
      
      // 更新当前配置
      currentConfig.value = config;
      
      console.log('加载配置成功:', JSON.stringify(currentConfig.value));
    } catch (error) {
      console.error('加载配置失败:', error);
      showError('加载配置失败: ' + formatError(error));
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
      showSuccess('保存配置成功');
    });
  } catch (error) {
    console.error('保存配置失败:', error);
    showError('保存配置失败: ' + formatError(error));
  }
};

const startTask = async () => {
  if (!canStartTask.value) {
    showWarning('请选择要复制的表');
    return;
  }

  await runWithLoading(async () => {
    try {
      await taskApi.start(currentConfig.value);
      showSuccess('任务创建成功');
      
      // 跳转到任务监控页面
      router.push('/');
    } catch (error) {
      showError('创建任务失败: ' + formatError(error));
    }
  });
};

const goBack = () => {
  router.push('/config');
};

onMounted(() => {
  console.log('ConfigDetail组件挂载，isCreating:', props.isCreating, 'configName:', props.configName);
  
  if (!props.isCreating && props.configName) {
    console.log('开始加载配置:', props.configName);
    loadConfig();
  } else {
    console.log('创建新配置');
    // 确保初始化默认值
    currentConfig.value = initConfig();
    console.log('初始化配置:', JSON.stringify(currentConfig.value));
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