// 数据库配置
export interface DatabaseConfig {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl_mode?: string;
  ssh_config?: SSHConfig;
}

// SSH配置
export interface SSHConfig {
  host: string;
  port: number;
  username: string;
  auth_type: 'password' | 'private_key';
  password?: string;
  private_key_path?: string;
  passphrase?: string;
}

export type TableInfoChangeStatus = 'Added' | 'Removed';

export interface MaskRule {
  rule_type: 'hash' | 'fixed' | 'pattern';
  pattern?: string;
}

export interface ColumnConfig {
  name: string;
  mask_rule?: MaskRule;
  ignore: boolean;
  status?: TableInfoChangeStatus;
}

// 表配置
export interface TableConfig {
  name: string;
  columns: ColumnConfig[];
  structure_only: boolean;
  ignore_foreign_keys: boolean;
  ignore: boolean;
  status?: TableInfoChangeStatus;
}

// 完整配置
export interface Config {
  name: string;
  source_db: DatabaseConfig;
  target_db: DatabaseConfig;
  tables: TableConfig [];
}

export interface Progress {
  current: number;
  total: number;
  table_name: string;
}

export type TaskState = 'pending' | 'running' | 'completed' | 'failed';

export interface TaskStatus {
  id: string;
  status: TaskState;
  start_time?: string;
  end_time?: string;
  message?: string;
  progress?: Progress;
}

// 任务
export interface Task {
  id: string;
  config: Config;
  status: TaskStatus;
} 