export interface SSHConfig {
  host: string;
  port: number;
  username: string;
  auth_type: 'password' | 'key';
  password?: string;
  private_key?: string;
}

export interface DatabaseConfig {
  type: 'mysql' | 'postgresql';
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl_mode?: 'disable' | 'require';
}

export interface MaskRule {
  column: string;
  rule_type: 'hash' | 'fixed' | 'pattern' | '';
  pattern?: string;
}

export interface TableConfig {
  name: string;
  columns: string[];
  mask_rules: MaskRule[];
}

export interface Config {
  name: string;
  source_db: DatabaseConfig;
  target_db: DatabaseConfig;
  source_ssh?: SSHConfig;
  target_ssh?: SSHConfig;
  tables: (string | TableConfig)[];
}

export interface Task {
  id: string;
  config: Config;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  error?: string;
  created_at: string;
  updated_at: string;
}

export type TaskState = 'pending' | 'running' | 'completed' | 'failed';

export interface TaskStatus {
  id: string;
  status: TaskState;
  start_time?: string;
  end_time?: string;
  message?: string;
  progress?: {
    current: number;
    total: number;
    table_name: string;
  };
} 