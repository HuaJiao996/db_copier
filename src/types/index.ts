

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
  private_key?: string;
  passphrase?: string;
}

// 掩码规则
export interface MaskRule {
  column: string;
  rule_type: string;
  pattern?: string;
}

// 表配置
export interface TableConfig {
  name: string;
  columns: string[];
  mask_rules: MaskRule[];
  structure_only?: boolean;
  ignore_foreign_keys?: boolean;
  last_updated?: string;
}

// 完整配置
export interface Config {
  name: string;
  source_db: DatabaseConfig;
  target_db: DatabaseConfig;
  tables: TableConfig [];
}

// 任务状态
export interface TaskStatus {
  id: string;
  status: string;
  progress: number;
  current_table?: string;
  current_operation?: string;
  error?: string;
  start_time: string;
  end_time?: string;
  total_rows?: number;
  processed_rows?: number;
}

// 任务
export interface Task {
  id: string;
  config: Config;
  status: TaskStatus;
} 