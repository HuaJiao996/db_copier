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
import type { Config } from '@/types'
import { ElMessage } from 'element-plus'
import { configApi } from '@/services/api'

const props = defineProps<{
  modelValue: boolean
  config: Config
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'config-loaded', config: Config): void
  (e: 'saved', configName: string): void
}>()

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const activeTab = ref('save')
const configName = ref('')
const configList = ref<{ name: string }[]>([])
const loading = ref(false)

const loadConfigs = async () => {
  try {
    loading.value = true
    const configs = await configApi.list()
    configList.value = configs.map((name: string) => ({ name }))
  } catch (error) {
    console.error('加载配置列表失败:', error)
    ElMessage.error('加载配置列表失败')
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  if (!configName.value) {
    ElMessage.error('请输入配置名称')
    return
  }
  
  try {
    loading.value = true
    await configApi.save(props.config)
    ElMessage.success('保存配置成功')
    emit('saved', configName.value)
    visible.value = false
    await loadConfigs()
  } catch (error) {
    console.error('保存配置失败:', error)
    ElMessage.error('保存配置失败')
  } finally {
    loading.value = false
  }
}

const loadConfig = async (name: string) => {
  try {
    loading.value = true
    const config = await configApi.load(name)
    emit('config-loaded', config)
    visible.value = false
    ElMessage.success('加载成功')
  } catch (error) {
    console.error('加载配置失败:', error)
    ElMessage.error('加载配置失败')
  } finally {
    loading.value = false
  }
}

const deleteConfig = async (name: string) => {
  try {
    loading.value = true
    await configApi.delete(name)
    ElMessage.success('删除配置成功')
    await loadConfigs()
  } catch (error) {
    console.error('删除配置失败:', error)
    ElMessage.error('删除配置失败')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadConfigs()
})
</script> 