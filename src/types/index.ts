

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

// 掩码规则
export interface MaskRule {
  rule_type: string;
  pattern?: string;
}

export interface ColumnConfig {
  name: string;
  mask_rule?: MaskRule;
  ignore: boolean;
}
// 表配置
export interface TableConfig {
  name: string;
  columns: ColumnConfig[];
  structure_only?: boolean;
  ignore_foreign_keys?: boolean;
  ignore: boolean;
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
  table_name?: string;
}
// 任务状态
export interface TaskStatus {
  id: string;
  status: string;
  progress: Progress;
  current_table?: string;
  current_operation?: string;
  error?: string;
  start_time: string;
  end_time?: string;
  total_rows?: number;
  processed_rows?: number;
  message?: string;
}

// 任务
export interface Task {
  id: string;
  config: Config;
  status: TaskStatus;
} 