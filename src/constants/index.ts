/**
 * 应用常量
 */

// 任务状态
export enum TaskStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed'
}

// 数据库类型
export enum DatabaseType {
  MYSQL = 'mysql',
  POSTGRESQL = 'postgresql',
  SQLSERVER = 'sqlserver'
}

// 路由路径
export const ROUTES = {
  CONFIG_MANAGER: '/',
  CONFIG_DETAIL: '/config/:id',
  TASK_MANAGER: '/tasks'
};

// 本地存储键
export const STORAGE_KEYS = {
  CONFIGS: 'db_copier_configs',
  TASKS: 'db_copier_tasks'
};

// API错误消息
export const ERROR_MESSAGES = {
  CONNECTION_FAILED: '连接失败，请检查配置',
  TASK_CREATION_FAILED: '创建任务失败',
  CONFIG_NOT_FOUND: '未找到配置'
}; 