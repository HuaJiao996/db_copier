<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, CaretRight, Document, DocumentCopy } from '@element-plus/icons-vue';
import { useRouter } from 'vue-router';
import type { Config } from '../types';

const router = useRouter();
const loading = ref(false);
const configList = ref<{ name: string }[]>([]);
const selectedConfigs = ref<{ name: string }[]>([]);

const loadConfigs = async () => {
  try {
    loading.value = true;
    const configs = await invoke<string[]>('list_configs');
    configList.value = configs.map(name => ({ name }));
  } catch (error) {
    ElMessage.error('加载配置失败: ' + error);
  } finally {
    loading.value = false;
  }
};

const createConfig = () => {
  router.push('/config/new');
};

const handleConfigClick = (row: { name: string }) => {
  router.push(`/config/${row.name}`);
};

const handleConfigDelete = async (name: string) => {
  try {
    await ElMessageBox.confirm('确定要删除该配置吗？', '提示', {
      type: 'warning',
    });
    await invoke('delete_config', { name });
    ElMessage.success('删除配置成功');
    await loadConfigs();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除配置失败: ' + error);
    }
  }
};

const startTask = async (name: string) => {
  try {
    loading.value = true;
    // 先加载配置
    const config = await invoke<Config>('load_config', { name });
    // 启动任务
    const taskId = await invoke<string>('start_copy', { config });
    ElMessage.success({
      message: '任务创建成功',
      duration: 2000
    });
    // 跳转到任务监控页面，并传递任务ID
    router.push({
      path: '/',
      query: { taskId }
    });
  } catch (error) {
    ElMessage.error({
      message: '创建任务失败: ' + error,
      duration: 5000
    });
  } finally {
    loading.value = false;
  }
};

const copyConfig = async (name: string) => {
  try {
    // 弹出输入框让用户输入新配置名称
    const { value: newName } = await ElMessageBox.prompt('请输入新配置名称', '复制配置', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputPattern: /^[a-zA-Z0-9_\u4e00-\u9fa5]{2,50}$/,
      inputErrorMessage: '配置名称必须是2-50个字符（支持中文、字母、数字、下划线）'
    });
    
    if (newName) {
      loading.value = true;
      // 加载原配置
      const config = await invoke<Config>('load_config', { name });
      // 保存为新配置
      await invoke('save_config', { 
        name: newName, 
        config: { ...config, name: newName }
      });
      ElMessage.success('复制配置成功');
      await loadConfigs();
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('复制配置失败: ' + error);
    }
  } finally {
    loading.value = false;
  }
};

const startBatchTasks = async () => {
  if (selectedConfigs.value.length === 0) {
    ElMessage.warning('请选择要启动的配置');
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定要启动选中的 ${selectedConfigs.value.length} 个配置吗？`,
      '批量启动任务',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    loading.value = true;
    const results = await Promise.allSettled(
      selectedConfigs.value.map(async (config) => {
        try {
          const loadedConfig = await invoke<Config>('load_config', { name: config.name });
          const taskId = await invoke<string>('start_copy', { config: loadedConfig });
          return { name: config.name, success: true, taskId };
        } catch (error) {
          return { name: config.name, success: false, error };
        }
      })
    );

    const succeeded = results
      .filter((r): r is PromiseFulfilledResult<{ name: string; success: true; taskId: string }> => 
        r.status === 'fulfilled' && r.value.success
      );
    const failed = results.filter(r => 
      r.status === 'fulfilled' && !r.value.success
    ).length;

    if (succeeded.length > 0) {
      ElMessage.success(`成功启动 ${succeeded.length} 个任务`);
      // 跳转到任务监控页面，并传递所有成功的任务ID
      router.push({
        path: '/',
        query: { 
          taskId: succeeded.map(r => r.value.taskId).join(',')
        }
      });
    }
    if (failed > 0) {
      ElMessage.error(`${failed} 个任务启动失败`);
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('批量启动任务失败');
    }
  } finally {
    loading.value = false;
  }
};

const handleSelectionChange = (selection: { name: string }[]) => {
  selectedConfigs.value = selection;
};

onMounted(() => {
  loadConfigs();
});
</script>

<template>
  <div class="config-manager">
    <div class="header">
      <h2>配置管理</h2>
      <el-button-group>
        <el-button type="primary" @click="createConfig">
          <el-icon><Plus /></el-icon>
          新建配置
        </el-button>
        <el-button 
          type="success" 
          @click="startBatchTasks"
          :disabled="selectedConfigs.length === 0"
        >
          <el-icon><CaretRight /></el-icon>
          批量启动
        </el-button>
      </el-button-group>
    </div>

    <el-card class="config-list" v-loading="loading">
      <template #header>
        <div class="card-header">
          <span>配置列表</span>
          <span class="selection-info" v-if="selectedConfigs.length > 0">
            已选择 {{ selectedConfigs.length }} 项
          </span>
        </div>
      </template>

      <el-table
        :data="configList"
        style="width: 100%"
        @row-click="handleConfigClick"
        @selection-change="handleSelectionChange"
      >
        <el-table-column type="selection" width="55" />
        <el-table-column prop="name" label="配置名称" />
        <el-table-column label="操作" width="360" align="right">
          <template #default="{ row }">
            <el-button 
              link 
              type="primary" 
              @click.stop="handleConfigClick(row)"
            >
              <el-icon><Document /></el-icon>
              编辑
            </el-button>
            <el-button 
              link 
              type="info" 
              @click.stop="copyConfig(row.name)"
            >
              <el-icon><DocumentCopy /></el-icon>
              复制
            </el-button>
            <el-button 
              link 
              type="success" 
              @click.stop="startTask(row.name)"
            >
              <el-icon><CaretRight /></el-icon>
              启动任务
            </el-button>
            <el-button 
              link 
              type="danger" 
              @click.stop="handleConfigDelete(row.name)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <el-empty
        v-if="configList.length === 0"
        description="暂无配置"
      >
        <el-button type="primary" @click="createConfig">
          <el-icon><Plus /></el-icon>
          创建配置
        </el-button>
      </el-empty>
    </el-card>
  </div>
</template>

<style scoped>
.config-manager {
  padding: 20px;
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

.config-list {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

:deep(.el-table__row) {
  cursor: pointer;
}

:deep(.el-table__row:hover) {
  background-color: var(--el-fill-color-light) !important;
}

:deep(.el-button--success.is-link) {
  --el-button-hover-text-color: var(--el-color-success-light-3);
}

:deep(.el-button--success.is-link:hover) {
  color: var(--el-color-success-light-3);
}

.selection-info {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

:deep(.el-button--info.is-link) {
  --el-button-hover-text-color: var(--el-color-info-light-3);
}

:deep(.el-button--info.is-link:hover) {
  color: var(--el-color-info-light-3);
}

.el-button-group {
  display: flex;
  gap: 8px;
}
</style> 