<template>
  <div>
    <el-form-item :label="label">
      <el-card>
        <el-form-item>
          <el-switch v-model="useSSH" :active-text="`启用${label}SSH隧道`" />
        </el-form-item>

        <template v-if="useSSH">
          <el-form-item prop="ssh.host" label="主机地址">
            <el-input v-model="config.host" />
          </el-form-item>
          
          <el-form-item prop="ssh.port" label="端口">
            <el-input-number v-model="config.port" :min="1" :max="65535" />
          </el-form-item>
          
          <el-form-item prop="ssh.username" label="用户名">
            <el-input v-model="config.username" />
          </el-form-item>
          
          <el-form-item label="认证方式">
            <el-radio-group v-model="authType">
              <el-radio label="key">密钥认证</el-radio>
              <el-radio label="password">密码认证</el-radio>
            </el-radio-group>
          </el-form-item>
          
          <el-form-item v-if="authType === 'key'" prop="ssh.private_key" label="密钥文件">
            <el-input v-model="config.private_key">
              <template #append>
                <el-button @click="selectKeyFile">选择文件</el-button>
              </template>
            </el-input>
          </el-form-item>
          
          <el-form-item v-else prop="ssh.password" label="密码">
            <el-input v-model="config.password" type="password" show-password />
          </el-form-item>
        </template>
      </el-card>
    </el-form-item>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import type { SSHConfig } from '../types'

const props = defineProps<{
  label: string
  modelValue?: SSHConfig
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: SSHConfig | undefined): void
}>()

const useSSH = ref(!!props.modelValue)
const authType = ref(props.modelValue?.private_key ? 'key' : 'password')
const config = ref<SSHConfig>(props.modelValue || {
  host: '',
  port: 22,
  username: '',
  auth_type: 'password',
  private_key: '',
  password: ''
})

watch(useSSH, (val) => {
  emit('update:modelValue', val ? config.value : undefined)
})

watch(config, (val) => {
  if (useSSH.value) {
    emit('update:modelValue', val)
  }
}, { deep: true })

const selectKeyFile = async () => {
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'SSH Key',
      extensions: ['pem', 'key']
    }]
  })
  
  if (selected && typeof selected === 'string') {
    config.value.private_key = selected
  }
}
</script> 