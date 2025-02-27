// 公共类型定义

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::database::TaskStatus;

/// 任务存储，用于在内存中跟踪任务状态
#[derive(Clone)]
pub struct TaskStore(pub Arc<Mutex<HashMap<String, TaskStatus>>>);

/// 配置摘要信息
#[derive(Debug, Serialize)]
pub struct ConfigSummary {
    pub source_db: String,
    pub target_db: String,
    pub table_count: usize,
    pub total_columns: usize,
    pub has_source_ssh: bool,
    pub has_target_ssh: bool,
} 