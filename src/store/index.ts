/**
 * 状态管理入口文件
 * 集中管理应用状态
 */

import { reactive, readonly } from 'vue';
import { Task } from '@/types';

// 定义状态接口
interface State {
  tasks: Task[];
  configs: any[];
  loading: boolean;
  currentConfig: any | null;
}

// 创建初始状态
const state = reactive<State>({
  tasks: [],
  configs: [],
  loading: false,
  currentConfig: null
});

// 定义操作方法
const actions = {
  // 设置任务列表
  setTasks(tasks: Task[]) {
    state.tasks = tasks;
  },
  
  // 设置配置列表
  setConfigs(configs: any[]) {
    state.configs = configs;
  },
  
  // 设置加载状态
  setLoading(loading: boolean) {
    state.loading = loading;
  },
  
  // 设置当前配置
  setCurrentConfig(config: any) {
    state.currentConfig = config;
  }
};

// 导出只读状态和操作方法
export default {
  state: readonly(state),
  actions
}; 