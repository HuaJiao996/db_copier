/**
 * API服务
 * 集中管理所有与后端的通信
 */

import { invoke } from "@tauri-apps/api/core";
import { DatabaseConfig, Config, TaskStatus } from '@/types';

/**
 * 配置相关API
 */
export const configApi = {
  /**
   * 获取配置列表
   * @returns 配置名称列表
   */
  async list(): Promise<string[]> {
    return await invoke<string[]>('list_configs');
  },

  /**
   * 加载配置
   * @param name 配置名称
   * @returns 配置对象
   */
  async load(name: string): Promise<Config> {
    return await invoke<Config>('load_config', { name });
  },

  /**
   * 保存配置
   * @param name 配置名称
   * @param config 配置对象
   */
  async save(config: Config): Promise<void> {
    await invoke('save_config', { config });
  },

  /**
   * 删除配置
   * @param name 配置名称
   */
  async delete(name: string): Promise<void> {
    await invoke('delete_config', { name });
  },

  /**
   * 导入配置
   * @param filePath 文件路径
   * @returns 导入的配置
   */
  async import(filePath: string): Promise<void> {
    await invoke<Config>('import_config', { filePath });
  },

  /**
   * 导出配置
   * @param name 配置名称
   * @param filePath 文件路径
   */
  async export(name: string, filePath: string): Promise<void> {
    await invoke('export_config', { name, filePath });
  }
};

/**
 * 任务相关API
 */
export const taskApi = {
  /**
   * 启动复制任务
   * @param config 配置对象
   * @returns 任务ID
   */
  async start(config: Config): Promise<string> {
    return await invoke<string>('start_copy', { config });
  },

  /**
   * 获取任务状态
   * @param taskId 任务ID
   * @returns 任务状态
   */
  async getStatus(taskId: string): Promise<TaskStatus> {
    return await invoke<TaskStatus>('get_task_status', { taskId });
  },

  /**
   * 停止任务
   * @param taskId 任务ID
   */
  async stop(taskId: string): Promise<void> {
    await invoke('stop_task', { taskId });
  }
};

/**
 * 数据库相关API
 */
export const databaseApi = {
  // 测试数据库连接
  testConnection: async (config: DatabaseConfig) => {
    try {
      
      console.log('发送测试连接请求:', config);
      
      return await invoke<string>('test_connection', { config });
    } catch (error) {
      console.error('测试连接失败:', error);
      throw error;
    }
  },
  
  // 获取数据库表
  getTables: (config: DatabaseConfig) => {
    return invoke<string[]>('get_tables', { config });
  },
  
  // 获取表的列
  getTableColumns: (config: DatabaseConfig, tableName: string) => {
    return invoke<string[]>('get_table_columns', { config, tableName });
  },
};

