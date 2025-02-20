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
              v-model:db-config="currentConfig.source_db"
              v-model:ssh-config="currentConfig.source_ssh"
              v-model:enableSSH="enableSourceSSH"
            />

            <!-- 目标数据库配置 -->
            <DatabaseConfig
              type="target"
              v-model:db-config="currentConfig.target_db"
              v-model:ssh-config="currentConfig.target_ssh"
              v-model:enableSSH="enableTargetSSH"
            />
          </div>
        </el-tab-pane>

        <el-tab-pane label="表和列配置" name="tables">
          <!-- 表配置 -->
          <TableConfig
            :config="currentConfig"
            :loading="loading"
            v-model:selected-tables="currentConfig.tables"
            @start-task="startTask"
          />
        </el-tab-pane>
      </el-tabs>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import type { Config, TableConfig as TableConfigType } from '../types';
import DatabaseConfig from '../components/DatabaseConfig.vue';
import TableConfig from '../components/TableConfig.vue';
import { useRouter, useRoute } from 'vue-router';

const props = defineProps<{
  isCreating: boolean;
  configName?: string;
}>();

const router = useRouter();
const route = useRoute();
const formRef = ref<FormInstance>();
const activeTab = ref('connection');
const loading = ref(false);

const initConfig = (): Config => ({
  name: props.configName || '',
  source_db: {
    type: 'postgresql',
    host: '',
    port: 5432,
    database: '',
    username: '',
    password: '',
    ssl_mode: 'require'
  },
  target_db: {
    type: 'postgresql',
    host: '',
    port: 5432,
    database: '',
    username: '',
    password: '',
    ssl_mode: 'require'
  },
  source_ssh: undefined,
  target_ssh: undefined,
  tables: []
});

const currentConfig = ref<Config>(initConfig());
const enableSourceSSH = ref(false);
const enableTargetSSH = ref(false);

const rules = {
  name: [
    { required: true, message: '请输入配置名称', trigger: 'blur' },
    { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' }
  ],
} as FormRules;

const canStartTask = computed(() => {
  return currentConfig.value.tables && currentConfig.value.tables.length > 0;
});

const loadConfig = async () => {
  if (!props.configName) return;
  
  try {
    loading.value = true;
    const config = await invoke<Config>('load_config', { name: props.configName });
    currentConfig.value = {
      ...config,
      name: props.configName // 确保名称正确
    };
    enableSourceSSH.value = !!config.source_ssh;
    enableTargetSSH.value = !!config.target_ssh;
  } catch (error) {
    ElMessage.error('加载配置失败: ' + error);
    router.push('/config');
  } finally {
    loading.value = false;
  }
};

const saveConfig = async () => {
  if (!formRef.value) return;

  try {
    await formRef.value.validate();
    loading.value = true;
    
    // 根据 SSH 启用状态设置配置
    const configToSave = { ...currentConfig.value };
    if (!enableSourceSSH.value) {
      configToSave.source_ssh = undefined;
    }
    if (!enableTargetSSH.value) {
      configToSave.target_ssh = undefined;
    }

    // 确保表配置格式正确
    configToSave.tables = configToSave.tables.map(table => {
      if (typeof table === 'string') {
        return {
          name: table,
          columns: [],
          mask_rules: []
        };
      }
      return table;
    });
    
    await invoke('save_config', { 
      name: configToSave.name, 
      config: configToSave 
    });
    
    ElMessage.success('保存配置成功');
  } catch (error) {
    console.error('保存配置失败:', error);
    if (error instanceof Error) {
      ElMessage.error(error.message);
    } else {
      ElMessage.error('保存配置失败，请检查表单填写是否正确');
    }
  } finally {
    loading.value = false;
  }
};

const startTask = async () => {
  if (!canStartTask.value) {
    ElMessage.warning('请选择要复制的表');
    return;
  }

  try {
    loading.value = true;
    const taskId = await invoke<string>('start_copy', { config: currentConfig.value });
    ElMessage.success({
      message: '任务创建成功',
      duration: 2000
    });
    
    // 跳转到任务监控页面
    router.push('/');
  } catch (error) {
    ElMessage.error({
      message: '创建任务失败: ' + error,
      duration: 5000
    });
  } finally {
    loading.value = false;
  }
};

const goBack = () => {
  router.push('/config');
};

onMounted(() => {
  if (!props.isCreating && props.configName) {
    loadConfig();
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