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

    <!-- SSH 配置 -->
    <div class="ssh-config">
      <div class="ssh-header">
        <el-checkbox v-model="enableSSH">启用 SSH 隧道</el-checkbox>
      </div>

      <div v-if="enableSSH" class="ssh-form">
        <el-form-item label="SSH 主机">
          <el-input v-model="sshConfig.host" placeholder="SSH 服务器地址" />
        </el-form-item>

        <el-form-item label="SSH 端口">
          <el-input-number 
            v-model="sshConfig.port" 
            :min="1" 
            :max="65535"
            :controls="false"
            style="width: 120px"
          />
        </el-form-item>

        <el-form-item label="SSH 用户名">
          <el-input v-model="sshConfig.username" />
        </el-form-item>

        <el-form-item label="认证方式">
          <el-radio-group v-model="sshConfig.auth_type">
            <el-radio label="password">密码</el-radio>
            <el-radio label="key">密钥</el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item 
          :label="sshConfig.auth_type === 'password' ? '密码' : '私钥'"
        >
          <el-input 
            v-if="sshConfig.auth_type === 'password'"
            v-model="sshConfig.password" 
            type="password" 
            show-password 
          />
          <el-input 
            v-else
            v-model="sshConfig.private_key" 
            type="textarea" 
            :rows="3"
            placeholder="请输入 SSH 私钥内容"
          />
        </el-form-item>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { DatabaseConfig, SSHConfig } from '../types';

const props = defineProps<{
  type: 'source' | 'target';
  dbConfig: DatabaseConfig;
  sshConfig?: SSHConfig;
  enableSSH: boolean;
}>();

const emit = defineEmits<{
  'update:dbConfig': [config: DatabaseConfig],
  'update:sshConfig': [config: SSHConfig],
  'update:enableSSH': [value: boolean],
}>();

const dbConfig = computed({
  get: () => ({
    ...props.dbConfig,
    type: 'postgresql' // 固定使用 PostgreSQL
  }),
  set: (value) => emit('update:dbConfig', {
    ...value,
    type: 'postgresql' // 确保类型始终为 PostgreSQL
  })
});

const sshConfig = computed({
  get: () => props.sshConfig || {
    host: '',
    port: 22,
    username: '',
    auth_type: 'password',
    password: '',
    private_key: ''
  },
  set: (value) => emit('update:sshConfig', value)
});

const enableSSH = computed({
  get: () => props.enableSSH,
  set: (value) => emit('update:enableSSH', value)
});
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