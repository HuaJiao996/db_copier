<template>
  <div class="task-manager">
    <div class="header">
      <h2>任务管理</h2>
      <el-button-group>
        <el-button type="primary" @click="refreshTasks">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button type="primary">
          <el-icon><Download /></el-icon>
          导出记录
        </el-button>
      </el-button-group>
    </div>

    <el-card class="task-list" v-loading="loading">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <span>任务列表</span>
            <el-tag 
              v-if="runningCount > 0" 
              type="success" 
              effect="plain" 
              size="small"
            >
              {{ runningCount }} 个运行中
            </el-tag>
          </div>
          <div class="filter-section">
            <el-select
              v-model="filterStatus"
              placeholder="状态筛选"
              clearable
              size="small"
              style="width: 120px"
            >
              <el-option label="运行中" value="running" />
              <el-option label="等待中" value="pending" />
              <el-option label="已完成" value="completed" />
              <el-option label="失败" value="failed" />
            </el-select>
            <el-input
              v-model="searchKeyword"
              placeholder="搜索任务ID"
              clearable
              size="small"
              style="width: 200px"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </div>
        </div>
      </template>

      <el-table :data="filteredTasks" style="width: 100%">
        <el-table-column prop="id" label="任务ID" width="180" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ getStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="progress" label="进度" width="200">
          <template #default="{ row }">
            <div v-if="row.progress">
              <el-progress 
                :percentage="Math.floor((row.progress.current / row.progress.total) * 100)"
                :status="row.status === 'failed' ? 'exception' : row.status === 'completed' ? 'success' : ''"
              />
              <div class="progress-detail">
                {{ row.progress.table_name }}
                ({{ row.progress.current }}/{{ row.progress.total }})
              </div>
            </div>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column prop="start_time" label="开始时间" width="180" />
        <el-table-column prop="end_time" label="结束时间" width="180" />
        <el-table-column prop="message" label="消息" show-overflow-tooltip />
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button 
              link 
              type="primary" 
              :disabled="row.status === 'running'"
              @click="showTaskDetail(row)"
            >
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="totalFilteredTasks"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 任务详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="任务详情"
      width="60%"
    >
      <div v-if="selectedTask" class="task-detail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="任务ID">{{ selectedTask.id }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(selectedTask.status)">
              {{ getStatusText(selectedTask.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="开始时间">{{ selectedTask.start_time || '-' }}</el-descriptions-item>
          <el-descriptions-item label="结束时间">{{ selectedTask.end_time || '-' }}</el-descriptions-item>
          <el-descriptions-item label="消息" :span="2">{{ selectedTask.message || '-' }}</el-descriptions-item>
        </el-descriptions>

        <div v-if="selectedTask.progress" class="task-progress">
          <h4>复制进度</h4>
          <el-progress 
            :percentage="Math.floor((selectedTask.progress.current / selectedTask.progress.total) * 100)"
            :status="selectedTask.status === 'failed' ? 'exception' : selectedTask.status === 'completed' ? 'success' : ''"
          />
          <p class="progress-info">
            当前表：{{ selectedTask.progress.table_name }}
            ({{ selectedTask.progress.current }}/{{ selectedTask.progress.total }})
          </p>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { Download, Refresh, Search } from '@element-plus/icons-vue';
import type { TaskStatus } from '../types';
import { useRoute } from 'vue-router';

const loading = ref(false);
const tasks = ref<TaskStatus[]>([]);
const currentPage = ref(1);
const pageSize = ref(10);
const detailDialogVisible = ref(false);
const selectedTask = ref<TaskStatus | null>(null);
const filterStatus = ref('');
const searchKeyword = ref('');
let refreshInterval: number | null = null;
const route = useRoute();

// 计算运行中的任务数量
const runningCount = computed(() => {
  return tasks.value.filter(task => 
    task.status === 'pending' || task.status === 'running'
  ).length;
});

// 根据筛选条件过滤任务
const filteredTasks = computed(() => {
  let filtered = tasks.value;
  
  // 状态筛选
  if (filterStatus.value) {
    filtered = filtered.filter(task => task.status === filterStatus.value);
  }
  
  // 关键字搜索
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    filtered = filtered.filter(task => 
      task.id.toLowerCase().includes(keyword) ||
      (task.message && task.message.toLowerCase().includes(keyword))
    );
  }
  
  // 分页
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return filtered.slice(start, end);
});

// 计算过滤后的总任务数
const totalFilteredTasks = computed(() => {
  let filtered = tasks.value;
  
  if (filterStatus.value) {
    filtered = filtered.filter(task => task.status === filterStatus.value);
  }
  
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    filtered = filtered.filter(task => 
      task.id.toLowerCase().includes(keyword) ||
      (task.message && task.message.toLowerCase().includes(keyword))
    );
  }
  
  return filtered.length;
});

// 获取状态类型
const getStatusType = (status: string) => {
  switch (status) {
    case 'running': return 'primary';
    case 'completed': return 'success';
    case 'failed': return 'danger';
    default: return 'info';
  }
};

// 获取状态文本
const getStatusText = (status: string) => {
  switch (status) {
    case 'running': return '运行中';
    case 'completed': return '已完成';
    case 'failed': return '失败';
    case 'pending': return '等待中';
    default: return '未知';
  }
};

// 刷新任务列表
const refreshTasks = async () => {
  try {
    loading.value = true;
    // 获取所有任务的状态
    const taskIds = tasks.value.map(task => task.id);
    const updatedTasks = await Promise.all(
      taskIds.map(async (id) => {
        try {
          const status = await invoke<TaskStatus>('get_task_status', { taskId: id });
          return status;
        } catch (error) {
          console.error(`获取任务 ${id} 状态失败:`, error);
          return null;
        }
      })
    );
    
    // 过滤掉无效的任务，并保持现有任务
    const validTasks = updatedTasks.filter((task): task is TaskStatus => task !== null);
    tasks.value = validTasks;
  } finally {
    loading.value = false;
  }
};

// 添加新任务到列表
const addTask = (taskId: string) => {
  const newTask: TaskStatus = {
    id: taskId,
    status: 'pending',
    start_time: undefined,
    end_time: undefined,
    message: undefined,
    progress: undefined
  };
  tasks.value.push(newTask);
};

// 监听路由变化，如果有新任务ID参数则添加到列表
watch(() => route.query.taskId, (newTaskId) => {
  if (newTaskId) {
    const taskIds = (newTaskId as string).split(',');
    taskIds.forEach(id => {
      if (!tasks.value.find(t => t.id === id)) {
        addTask(id);
      }
    });
  }
}, { immediate: true });

// 监听筛选条件变化，重置分页
watch([filterStatus, searchKeyword], () => {
  currentPage.value = 1;
});

// 显示任务详情
const showTaskDetail = (task: TaskStatus) => {
  selectedTask.value = task;
  detailDialogVisible.value = true;
};

// 处理分页大小变化
const handleSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1;
};

// 处理页码变化
const handleCurrentChange = (page: number) => {
  currentPage.value = page;
};

onMounted(() => {
  // 如果URL中有taskId参数，添加到任务列表
  const taskId = route.query.taskId;
  if (taskId) {
    const taskIds = (taskId as string).split(',');
    taskIds.forEach(id => {
      if (!tasks.value.find(t => t.id === id)) {
        addTask(id);
      }
    });
  }
  
  // 每5秒刷新一次运行中的任务状态
  refreshInterval = window.setInterval(() => {
    if (runningCount.value > 0) {
      refreshTasks();
    }
  }, 5000);
});

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
});
</script>

<style scoped>
.task-manager {
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

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-left span {
  font-size: 16px;
  font-weight: 500;
}

.filter-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-detail {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.pagination {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.task-detail {
  padding: 20px 0;
}

.task-progress {
  margin-top: 24px;
}

.task-progress h4 {
  margin: 0 0 16px;
  font-size: 16px;
  font-weight: 500;
}

.progress-info {
  margin: 8px 0 0;
  font-size: 14px;
  color: var(--el-text-color-regular);
}
</style> 