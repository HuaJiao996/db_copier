<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { open } from "@tauri-apps/plugin-dialog";
import { Plus, CaretRight, Document, DocumentCopy, Upload } from '@element-plus/icons-vue';
import { useRouter } from 'vue-router';
import { configApi, taskApi } from '@/services/api';
import { useLoading } from '@/composables/useLoading';
import { useNotification } from '@/composables/useNotification';
import { ElMessageBox } from 'element-plus';
import { useI18n } from 'vue-i18n';

const router = useRouter();
const { t } = useI18n();
const { isLoading: loading, runWithLoading } = useLoading();
const { showSuccess, showError, showWarning, showConfirm, showPrompt } = useNotification();
const configList = ref<{ name: string }[]>([]);
const selectedConfigs = ref<{ name: string }[]>([]);
const migrateLoading = ref(false);

const loadConfigs = async () => {
  await runWithLoading(async () => {
    try {
      const configs = await configApi.list();
      configList.value = configs.map(name => ({ name }));
    } catch (error) {
      showError(t('configManager.errors.loadFailed') + ': ' + error);
    }
  });
};

const createConfig = () => {
  router.push('/config/new');
};

const handleConfigClick = (row: { name: string }) => {
  router.push(`/config/${row.name}`);
};

const handleConfigDelete = async (name: string) => {
  try {
    const confirmed = await showConfirm(
      t('common.tips'),
      t('configManager.confirmDelete'),
      { type: 'warning' }
    );
    if (!confirmed) return;
    
    await configApi.delete(name);
    showSuccess(t('configManager.messages.deleteSuccess'));
    await loadConfigs();
  } catch (error) {
    showError(t('configManager.errors.deleteFailed') + ': ' + error);
  }
};

const startTask = async (name: string) => {
  await runWithLoading(async () => {
    try {
      const config = await configApi.load(name);
      const taskId = await taskApi.start(config);
      showSuccess(t('configManager.messages.taskCreated'));
      router.push({
        path: '/',
        query: { taskId }
      });
    } catch (error) {
      showError(t('configManager.errors.taskCreateFailed') + ': ' + error);
    }
  });
};

const copyConfig = async (name: string) => {
  try {
    const newName = await showPrompt(
      t('configManager.copyConfig'),
      t('configManager.enterNewName'),
      {
        inputPattern: /^[a-zA-Z0-9_\u4e00-\u9fa5]{2,50}$/,
        inputErrorMessage: t('configManager.errors.invalidName')
      }
    );
    
    if (newName) {
      await runWithLoading(async () => {
        const config = await configApi.load(name);
        await configApi.save(config);
        showSuccess(t('configManager.messages.copySuccess'));
        await loadConfigs();
      });
    }
  } catch (error) {
    showError(t('configManager.errors.copyFailed') + ': ' + error);
  }
};

const startBatchTasks = async () => {
  if (selectedConfigs.value.length === 0) {
    showWarning(t('configManager.errors.noConfigSelected'));
    return;
  }

  try {
    const confirmed = await showConfirm(
      t('configManager.batchStart'),
      t('configManager.confirmBatchStart', { count: selectedConfigs.value.length }),
      { type: 'warning' }
    );

    if (!confirmed) return;

    await runWithLoading(async () => {
      const results = await Promise.allSettled(
        selectedConfigs.value.map(async (config) => {
          try {
            const loadedConfig = await configApi.load(config.name);
            const taskId = await taskApi.start(loadedConfig);
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
        showSuccess(t('configManager.messages.batchStartSuccess', { count: succeeded.length }));
        router.push({
          path: '/',
          query: { 
            taskId: succeeded.map(r => r.value.taskId).join(',')
          }
        });
      }
      if (failed > 0) {
        showError(t('configManager.errors.batchStartPartialFailed', { count: failed }));
      }
    });
  } catch (error) {
    showError(t('configManager.errors.batchStartFailed'));
  }
};

const handleSelectionChange = (selection: { name: string }[]) => {
  selectedConfigs.value = selection;
};

const importConfig = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'JSON',
        extensions: ['json']
      }]
    });

    if (!selected) return;

    const files = Array.isArray(selected) ? selected : [selected];
    
    migrateLoading.value = true;
    let success = 0;
    let failed = 0;
    const messages = [];

    for (const filePath of files) {
      try {
        await configApi.import(filePath);
        success++;
        messages.push(t('configManager.messages.importSuccess', { path: filePath }));
      } catch (error) {
        failed++;
        messages.push(t('configManager.errors.importFailed', { path: filePath, error }));
      }
    }

    await ElMessageBox.alert(
      t('configManager.messages.importComplete', {
        success,
        failed,
        details: messages.join('\n')
      }),
      t('configManager.importResult'),
      {
        confirmButtonText: t('common.confirm')
      }
    );
  } catch (error) {
    showError(t('configManager.errors.importFailed', { error }));
  } finally {
    migrateLoading.value = false;
  }
};

onMounted(() => {
  loadConfigs();
});
</script>

<template>
  <div class="config-manager">
    <div class="header">
      <h2>{{ $t('configManager.title') }}</h2>
      <el-button-group>
        <el-button type="primary" @click="createConfig">
          <el-icon><Plus /></el-icon>
          {{ $t('configManager.newConfig') }}
        </el-button>
        <el-button 
          type="success" 
          @click="startBatchTasks"
          :disabled="selectedConfigs.length === 0"
        >
          <el-icon><CaretRight /></el-icon>
          {{ $t('configManager.batchStart') }}
        </el-button>
        <el-button 
          type="warning" 
          @click="importConfig"
          :loading="migrateLoading"
        >
          <el-icon><Upload /></el-icon>
          {{ $t('configManager.importConfig') }}
        </el-button>
      </el-button-group>
    </div>

    <el-card class="config-list" v-loading="loading">
      <template #header>
        <div class="card-header">
          <span>{{ $t('configManager.configList') }}</span>
          <span class="selection-info" v-if="selectedConfigs.length > 0">
            {{ $t('configManager.selectedCount', { count: selectedConfigs.length }) }}
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
        <el-table-column prop="name" :label="$t('configManager.columns.name')" />
        <el-table-column :label="$t('configManager.columns.actions')" width="360" align="right">
          <template #default="{ row }">
            <el-button 
              link 
              type="primary" 
              @click.stop="handleConfigClick(row)"
            >
              <el-icon><Document /></el-icon>
              {{ $t('common.edit') }}
            </el-button>
            <el-button 
              link 
              type="info" 
              @click.stop="copyConfig(row.name)"
            >
              <el-icon><DocumentCopy /></el-icon>
              {{ $t('common.copy') }}
            </el-button>
            <el-button 
              link 
              type="success" 
              @click.stop="startTask(row.name)"
            >
              <el-icon><CaretRight /></el-icon>
              {{ $t('configManager.startTask') }}
            </el-button>
            <el-button 
              link 
              type="danger" 
              @click.stop="handleConfigDelete(row.name)"
            >
              {{ $t('common.delete') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <el-empty
        v-if="configList.length === 0"
        :description="$t('configManager.noConfig')"
      >
        <el-button type="primary" @click="createConfig">
          <el-icon><Plus /></el-icon>
          {{ $t('configManager.createConfig') }}
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