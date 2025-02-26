<template>
  <div class="database-config">
    <h3>{{ type === 'source' ? '源数据库' : '目标数据库' }}</h3>
    
    <!-- 数据库配置 -->
    <el-form-item label="主机地址" required>
      <el-input v-model="dbConfig.host" placeholder="例如：localhost" />
    </el-form-item>

    <el-form-item label="端口" required>
      <el-input-number 
        v-model="dbConfig.port" 
        :min="1" 
        :max="65535"
        :controls="false"
        style="width: 120px"
      />
    </el-form-item>

    <el-form-item label="数据库名" required>
      <el-input v-model="dbConfig.database" />
    </el-form-item>

    <el-form-item label="用户名" required>
      <el-input v-model="dbConfig.username" />
    </el-form-item>

    <el-form-item label="密码" required>
      <el-input v-model="dbConfig.password" type="password" show-password />
    </el-form-item>

    <el-form-item label="SSL模式">
      <el-select v-model="dbConfig.ssl_mode" style="width: 100%">
        <el-option label="首选 (prefer)" value="prefer" />
        <el-option label="要求 (require)" value="require" />
        <el-option label="禁用 (disable)" value="disable" />
      </el-select>
    </el-form-item>

    <!-- 测试连接按钮 -->
    <el-form-item>
      <el-button 
        type="primary" 
        :loading="testingConnection" 
        @click="testConnection"
      >
        测试连接
      </el-button>
    </el-form-item>

    <!-- SSH 配置 -->
    <div class="ssh-config">
      <div class="ssh-header">
        <el-checkbox v-model="enableSSHModel">启用 SSH 隧道</el-checkbox>
      </div>

      <div v-if="enableSSHModel" class="ssh-form">
        <el-form-item label="SSH 主机">
          <el-input v-model="dbConfig.ssh_config!.host" placeholder="SSH 服务器地址" />
        </el-form-item>

        <el-form-item label="SSH 端口">
          <el-input-number 
            v-model="dbConfig.ssh_config!.port" 
            :min="1" 
            :max="65535"
            :controls="false"
            style="width: 120px"
          />
        </el-form-item>

        <el-form-item label="SSH 用户名">
          <el-input v-model="dbConfig.ssh_config!.username" />
        </el-form-item>

        <el-form-item label="认证方式">
          <el-radio-group v-model="dbConfig.ssh_config!.auth_type">
            <el-radio label="password">密码</el-radio>
            <el-radio label="private_key">密钥</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item 
          :label="dbConfig.ssh_config!.auth_type === 'password' ? '密码' : '私钥'"
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
              placeholder="请选择私钥文件"
              readonly
            >
              <template #append>
                <el-button @click="selectPrivateKey">
                  选择文件
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

const props = defineProps<{
  type: 'source' | 'target';
}>();

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
      ElMessage.error('主机地址不能为空');
      testingConnection.value = false;
      return;
    }
    
    if (!dbConfig.value.database) {
      ElMessage.error('数据库名不能为空');
      testingConnection.value = false;
      return;
    }
    
    if (!dbConfig.value.username) {
      ElMessage.error('用户名不能为空');
      testingConnection.value = false;
      return;
    }
    
    console.log(`测试数据库连接:`, JSON.stringify(dbConfig.value));
    
    // 调用测试连接API
    const result = await databaseApi.testConnection(dbConfig.value);
    ElMessage.success(`连接成功: ${result}`);
  } catch (error) {
    console.error(`${props.type === 'source' ? '源' : '目标'}数据库连接失败:`, error);
    ElMessage.error(`连接失败: ${error}`);
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
    ElMessage.error('选择私钥文件失败');
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