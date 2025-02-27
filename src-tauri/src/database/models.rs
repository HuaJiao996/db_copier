use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub private_key_path: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: String,
    pub ssh_config: Option<SSHConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnConfig {
    pub name: String,
    pub mask_rule: Option<MaskRule>,
    pub ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableConfig {
    pub name: String,
    pub columns: Vec<ColumnConfig>,
    pub structure_only: bool,
    pub ignore_foreign_keys: bool,
    pub ignore: bool,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct MaskRule {
    pub rule_type: MaskRuleType,
    pub pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MaskRuleType {
    #[serde(rename = "hash")]
    Hash,
    #[serde(rename = "fixed")]
    Fixed,
    #[serde(rename = "pattern")]
    Pattern,
}

impl Default for MaskRuleType {
    fn default() -> Self {
        Self::Hash
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
    pub source_db: DatabaseConfig,
    pub target_db: DatabaseConfig,
    pub tables: Vec<TableConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub table_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskStatus {
    pub id: String,
    pub status: TaskState,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub message: Option<String>,
    pub progress: Option<Progress>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TaskState {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
} 