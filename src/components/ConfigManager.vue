<template>
  <el-dialog
    v-model="visible"
    title="配置管理"
    width="500px"
  >
    <el-tabs v-model="activeTab">
      <el-tab-pane label="保存配置" name="save">
        <el-form>
          <el-form-item label="配置名称">
            <el-input v-model="configName" placeholder="请输入配置名称" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="visible = false">取消</el-button>
          <el-button type="primary" @click="saveConfig">保存</el-button>
        </template>
      </el-tab-pane>
      
      <el-tab-pane label="加载配置" name="load">
        <el-table :data="configList" style="width: 100%">
          <el-table-column prop="name" label="配置名称" />
          <el-table-column fixed="right" label="操作" width="120">
            <template #default="{ row }">
              <el-button link type="primary" @click="loadConfig(row.name)">
                加载
              </el-button>
              <el-button link type="danger" @click="deleteConfig(row.name)">
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { Config } from '../types'

const props = defineProps<{
  modelValue: boolean
  config: Config
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'config-loaded', config: Config): void
}>()

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const activeTab = ref('save')
const configName = ref('')
const configList = ref<{ name: string }[]>([])

const refreshConfigList = async () => {
  try {
    const configs = await invoke<string[]>('list_configs')
    configList.value = configs.map(name => ({ name }))
  } catch (error) {
    ElMessage.error('获取配置列表失败: ' + error)
  }
}

const saveConfig = async () => {
  if (!configName.value) {
    ElMessage.warning('请输入配置名称')
    return
  }
  
  try {
    await invoke('save_config', {
      name: configName.value,
      config: props.config
    })
    ElMessage.success('保存成功')
    configName.value = ''
    refreshConfigList()
  } catch (error) {
    ElMessage.error('保存失败: ' + error)
  }
}

const loadConfig = async (name: string) => {
  try {
    const config = await invoke<Config>('load_config', { name })
    emit('config-loaded', config)
    visible.value = false
    ElMessage.success('加载成功')
  } catch (error) {
    ElMessage.error('加载失败: ' + error)
  }
}

const deleteConfig = async (name: string) => {
  try {
    await invoke('delete_config', { name })
    ElMessage.success('删除成功')
    refreshConfigList()
  } catch (error) {
    ElMessage.error('删除失败: ' + error)
  }
}

onMounted(() => {
  refreshConfigList()
})
</script> 