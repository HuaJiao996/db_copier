<template>
  <div class="database-config">
    <h3>{{ type === 'source' ? t('databaseConfig.source') : t('databaseConfig.target') }}</h3>
    
    <!-- 数据库配置 -->
    <el-form-item :label="t('databaseConfig.host')" required>
      <el-input v-model="dbConfig.host" :placeholder="t('databaseConfig.hostPlaceholder')" />
    </el-form-item>

    <el-form-item :label="t('databaseConfig.port')" required>
      <el-input-number 
        v-model="dbConfig.port" 
        :min="1" 
        :max="65535"
        :controls="false"
        style="width: 120px"
      />
    </el-form-item>

    <el-form-item :label="t('databaseConfig.database')" required>
      <el-input v-model="dbConfig.database" />
    </el-form-item>

    <el-form-item :label="t('databaseConfig.username')" required>
      <el-input v-model="dbConfig.username" />
    </el-form-item>

    <el-form-item :label="t('databaseConfig.password')" required>
      <el-input v-model="dbConfig.password" type="password" show-password />
    </el-form-item>

    <el-form-item :label="t('databaseConfig.sslMode')">
      <el-select v-model="dbConfig.ssl_mode" style="width: 100%">
        <el-option :label="t('databaseConfig.sslModes.prefer')" value="prefer" />
        <el-option :label="t('databaseConfig.sslModes.require')" value="require" />
        <el-option :label="t('databaseConfig.sslModes.disable')" value="disable" />
      </el-select>
    </el-form-item>

    <!-- 测试连接按钮 -->
    <el-form-item>
      <el-button 
        type="primary" 
        :loading="testingConnection" 
        @click="testConnection"
      >
        {{ t('databaseConfig.testConnection') }}
      </el-button>
    </el-form-item>

    <!-- SSH 配置 -->
    <div class="ssh-config">
      <div class="ssh-header">
        <el-checkbox v-model="enableSSHModel">{{ t('databaseConfig.enableSSH') }}</el-checkbox>
      </div>

      <div v-if="enableSSHModel" class="ssh-form">
        <el-form-item :label="t('databaseConfig.sshHost')">
          <el-input v-model="dbConfig.ssh_config!.host" :placeholder="t('databaseConfig.sshHostPlaceholder')" />
        </el-form-item>

        <el-form-item :label="t('databaseConfig.sshPort')">
          <el-input-number 
            v-model="dbConfig.ssh_config!.port" 
            :min="1" 
            :max="65535"
            :controls="false"
            style="width: 120px"
          />
        </el-form-item>

        <el-form-item :label="t('databaseConfig.sshUsername')">
          <el-input v-model="dbConfig.ssh_config!.username" />
        </el-form-item>

        <el-form-item :label="t('databaseConfig.authType')">
          <el-radio-group v-model="dbConfig.ssh_config!.auth_type">
            <el-radio label="password">{{ t('databaseConfig.authTypes.password') }}</el-radio>
            <el-radio label="private_key">{{ t('databaseConfig.authTypes.privateKey') }}</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item 
          :label="dbConfig.ssh_config!.auth_type === 'password' ? t('databaseConfig.password') : t('databaseConfig.privateKey')"
        >
          <el-input 
            v-if="dbConfig.ssh_config!.auth_type === 'password'"
            v-model="dbConfig.ssh_config!.password" 
            type="password" 
            show-password 
          />
          <div v-else class="key-select">
            <el-input
              v-model="dbConfig.ssh_config!.private_key_path"
              :placeholder="t('databaseConfig.selectPrivateKeyPlaceholder')"
              readonly
            >
              <template #append>
                <el-button @click="selectPrivateKey">
                  {{ t('databaseConfig.selectFile') }}
                </el-button>
              </template>
            </el-input>
          </div>
        </el-form-item>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { DatabaseConfig } from '@/types';
import { databaseApi } from '@/services/api';
import { ElMessage } from 'element-plus';
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  type: 'source' | 'target';
}>();

const { t } = useI18n();

// 使用defineModel来简化v-model绑定
const dbConfig = defineModel<DatabaseConfig>({ 
  default: () => ({
    host: '',
    port: 5432,
    database: '',
    username: '',
    password: '',
    ssl_mode: 'prefer'
  })
});

const enableSSHModel = computed({
  get: () => !!dbConfig.value.ssh_config,
  set: (val) => {
    console.log('enableSSHModel', val); 
    if (!val) {
      dbConfig.value.ssh_config = undefined;
      return;
    }
    dbConfig.value.ssh_config = {
        host: '',
        port: 22,
        username: '',
        auth_type: 'password',
        password: '',
        private_key_path: ''
      };
  }});

// 测试连接相关
const testingConnection = ref(false);

// 测试数据库连接
const testConnection = async () => {
  testingConnection.value = true;
  
  try {
    // 验证必填字段
    if (!dbConfig.value.host) {
      ElMessage.error(t('databaseConfig.errors.hostRequired'));
      testingConnection.value = false;
      return;
    }
    
    if (!dbConfig.value.database) {
      ElMessage.error(t('databaseConfig.errors.databaseRequired'));
      testingConnection.value = false;
      return;
    }
    
    if (!dbConfig.value.username) {
      ElMessage.error(t('databaseConfig.errors.usernameRequired'));
      testingConnection.value = false;
      return;
    }
    
    console.log(`测试数据库连接:`, JSON.stringify(dbConfig.value));
    
    // 调用测试连接API
    const result = await databaseApi.testConnection(dbConfig.value);
    ElMessage.success(t('databaseConfig.messages.connectionSuccess', { message: result }));
  } catch (error) {
    console.error(`${props.type === 'source' ? '源' : '目标'}数据库连接失败:`, error);
    ElMessage.error(t('databaseConfig.errors.connectionFailed', { error }));
  } finally {
    testingConnection.value = false;
  }
};

// 选择私钥文件
const selectPrivateKey = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'SSH Private Key',
        extensions: ['pem', 'key', 'ppk']
      }]
    });
    
    if (selected && typeof selected === 'string') {
      dbConfig.value.ssh_config!.private_key_path = selected;
    }
  } catch (error) {
    console.error('选择私钥文件失败:', error);
    ElMessage.error(t('databaseConfig.errors.selectKeyFailed'));
  }
};

</script>

<style scoped>
.database-config {
  padding: 24px;
  border: 1px solid var(--el-border-color-light);
  border-radius: 8px;
  background-color: var(--el-bg-color);
}

.database-config h3 {
  margin: 0 0 24px;
  font-size: 16px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.ssh-config {
  margin-top: 32px;
}

.ssh-header {
  margin-bottom: 24px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.ssh-form {
  padding: 16px;
  background-color: var(--el-fill-color-light);
  border-radius: 4px;
}

:deep(.el-form-item:last-child) {
  margin-bottom: 0;
}
</style> 