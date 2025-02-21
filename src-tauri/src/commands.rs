use crate::models::{Config, TaskStatus, TaskState, Progress, MaskRule, SSHConfig, DatabaseConfig, TableConfig};
use crate::db::{DbCopier, TableInfo};
use crate::monitor::MemoryMonitor;
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Local;
use tauri::State;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use log::{info, error};
use serde::Deserialize;

#[derive(Clone)]
pub struct TaskStore(pub Arc<Mutex<HashMap<String, TaskStatus>>>);

#[tauri::command]
pub async fn test_connection(config: Config) -> Result<String, String> {
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    match DbCopier::new(&config, memory_monitor).await {
        Ok(_) => Ok("连接成功".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn start_copy(
    config: Config,
    task_store: State<'_, TaskStore>,
) -> Result<String, String> {
    let task_id = Local::now().format("%Y%m%d%H%M%S").to_string();
    info!("Starting copy task: {}", task_id);
    
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    
    // 初始化任务状态
    {
        let mut tasks = task_store.0.lock().unwrap();
        tasks.insert(
            task_id.clone(),
            TaskStatus {
                id: task_id.clone(),
                status: TaskState::Pending,
                start_time: None,
                end_time: None,
                message: None,
                progress: None,
            },
        );
    }

    tokio::spawn({
        let task_store = task_store.inner().clone();
        let task_id = task_id.clone();
        let config = config.clone();
        
        async move {
            {
                let mut tasks = task_store.0.lock().unwrap();
                let task = tasks.get_mut(&task_id).unwrap();
                task.status = TaskState::Running;
                task.start_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                task.progress = Some(Progress {
                    current: 0,
                    total: config.tables.len(),
                    table_name: String::new(),
                });
            }

            match DbCopier::new(&config, memory_monitor).await {
                Ok(copier) => {
                    for (i, table) in config.tables.iter().enumerate() {
                        info!("Copying table {}: {}", i + 1, table.name);
                        
                        {
                            let mut tasks = task_store.0.lock().unwrap();
                            let task = tasks.get_mut(&task_id).unwrap();
                            task.progress.as_mut().unwrap().current = i;
                            task.progress.as_mut().unwrap().table_name = table.name.clone();
                        }

                        if let Err(e) = copier.copy_table(table).await {
                            error!("Failed to copy table {}: {}", table.name, e);
                            let mut tasks = task_store.0.lock().unwrap();
                            let task = tasks.get_mut(&task_id).unwrap();
                            task.status = TaskState::Failed;
                            task.end_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                            task.message = Some(e.to_string());
                            return;
                        }
                    }

                    let mut tasks = task_store.0.lock().unwrap();
                    let task = tasks.get_mut(&task_id).unwrap();
                    task.status = TaskState::Completed;
                    task.end_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                    task.message = Some("复制完成".to_string());
                    task.progress.as_mut().unwrap().current = config.tables.len();
                    info!("Copy task completed: {}", task_id);
                }
                Err(e) => {
                    error!("Failed to initialize copier: {}", e);
                    let mut tasks = task_store.0.lock().unwrap();
                    let task = tasks.get_mut(&task_id).unwrap();
                    task.status = TaskState::Failed;
                    task.end_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                    task.message = Some(e.to_string());
                }
            }
        }
    });

    Ok(task_id)
}

#[tauri::command]
pub fn get_task_status(
    task_id: String,
    task_store: State<'_, TaskStore>,
) -> Result<TaskStatus, String> {
    let tasks = task_store.0.lock().unwrap();
    tasks
        .get(&task_id)
        .cloned()
        .ok_or_else(|| "任务不存在".to_string())
}

#[tauri::command]
pub async fn get_table_columns(
    config: Config,
    table_name: String
) -> Result<Vec<String>, String> {
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| e.to_string())?;
    
    copier.get_table_columns(&table_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tables(config: Config) -> Result<Vec<String>, String> {
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| e.to_string())?;
    
    copier.get_tables()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(
    name: String,
    config: Config,
    storage: State<'_, Arc<Storage>>,
) -> Result<(), String> {
    storage.save_config(&name, &config)
        .await
        .map_err(|e| format!("保存配置失败: {}", e))
}

#[tauri::command]
pub async fn load_config(
    name: String,
    storage: State<'_, Arc<Storage>>,
) -> Result<Config, String> {
    match storage.load_config(&name).await {
        Ok(Some(config)) => Ok(config),
        Ok(None) => Err("配置不存在".to_string()),
        Err(e) => Err(format!("加载配置失败: {}", e)),
    }
}

#[tauri::command]
pub async fn list_configs(
    storage: State<'_, Arc<Storage>>,
) -> Result<Vec<String>, String> {
    storage.list_configs()
        .await
        .map_err(|e| format!("获取配置列表失败: {}", e))
}

#[tauri::command]
pub async fn delete_config(
    name: String,
    storage: State<'_, Arc<Storage>>,
) -> Result<(), String> {
    match storage.delete_config(&name).await {
        Ok(true) => Ok(()),
        Ok(false) => Err("配置不存在".to_string()),
        Err(e) => Err(format!("删除配置失败: {}", e)),
    }
}

#[tauri::command]
pub async fn get_table_info(
    config: Config,
    table_name: String,
) -> Result<TableInfo, String> {
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| e.to_string())?;
    
    copier.get_table_info(&table_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_table_structure(
    config: Config,
    table_info: TableInfo,
) -> Result<(), String> {
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| e.to_string())?;
    
    copier.sync_table_structure(&table_info, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_memory_usage(memory_monitor: tauri::State<'_, Arc<MemoryMonitor>>) -> Result<usize, String> {
    Ok(memory_monitor.get_usage())
}

#[tauri::command]
pub async fn migrate_configs(
    storage: State<'_, Arc<Storage>>,
) -> Result<String, String> {
    // 检查旧的配置目录
    let config_dir = PathBuf::from("configs");
    if !config_dir.exists() {
        return Ok("无需迁移：旧配置目录不存在".to_string());
    }

    let mut migrated = 0;
    let mut failed = 0;
    let mut messages = Vec::new();

    // 读取所有JSON文件
    for entry in fs::read_dir(&config_dir).map_err(|e| format!("读取配置目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取配置文件失败: {}", e))?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_name = path.file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| "无效的文件名".to_string())?;

            match migrate_single_config(&path, file_name, &storage).await {
                Ok(_) => {
                    migrated += 1;
                    messages.push(format!("成功迁移配置: {}", file_name));
                }
                Err(e) => {
                    failed += 1;
                    messages.push(format!("迁移配置失败 {}: {}", file_name, e));
                }
            }
        }
    }

    // 如果所有配置都迁移成功，重命名旧目录
    if failed == 0 && migrated > 0 {
        let backup_dir = PathBuf::from("configs_backup");
        if let Err(e) = fs::rename(&config_dir, &backup_dir) {
            messages.push(format!("警告：无法备份旧配置目录: {}", e));
        } else {
            messages.push("已将旧配置目录重命名为 'configs_backup'".to_string());
        }
    }

    let summary = format!(
        "迁移完成。成功: {}, 失败: {}\n{}",
        migrated,
        failed,
        messages.join("\n")
    );

    Ok(summary)
}

async fn migrate_single_config(
    path: &PathBuf,
    name: &str,
    storage: &Storage,
) -> Result<(), String> {
    // 读取JSON文件
    let config_json = fs::read_to_string(path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 尝试解析为旧格式
    #[derive(Debug, Deserialize)]
    struct OldTableConfig {
        name: String,
        columns: Vec<String>,
        mask_rules: Vec<MaskRule>,
    }

    #[derive(Debug, Deserialize)]
    struct OldConfig {
        source_ssh: Option<SSHConfig>,
        target_ssh: Option<SSHConfig>,
        source_db: DatabaseConfig,
        target_db: DatabaseConfig,
        tables: Vec<OldTableConfig>,
    }

    // 尝试解析配置
    let config = match serde_json::from_str::<Config>(&config_json) {
        Ok(config) => config,
        Err(_) => {
            // 尝试解析旧格式
            let old_config: OldConfig = serde_json::from_str(&config_json)
                .map_err(|e| format!("解析配置失败: {}", e))?;
            
            // 转换为新格式
            Config {
                source_ssh: old_config.source_ssh,
                target_ssh: old_config.target_ssh,
                source_db: old_config.source_db,
                target_db: old_config.target_db,
                tables: old_config.tables.into_iter().map(|t| TableConfig {
                    name: t.name,
                    columns: t.columns,
                    mask_rules: t.mask_rules,
                    structure_only: false,
                    ignore_foreign_keys: false,
                }).collect(),
            }
        }
    };

    // 保存到SQLite
    storage.save_config(name, &config)
        .await
        .map_err(|e| format!("保存到数据库失败: {}", e))
}

#[tauri::command]
pub async fn import_config(
    file_path: String,
) -> Result<Config, String> {
    // 读取JSON文件
    let config_json = fs::read_to_string(file_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 尝试解析为旧格式
    #[derive(Debug, Deserialize)]
    struct OldTableConfig {
        name: String,
        columns: Vec<String>,
        mask_rules: Vec<MaskRule>,
    }

    #[derive(Debug, Deserialize)]
    struct OldConfig {
        source_ssh: Option<SSHConfig>,
        target_ssh: Option<SSHConfig>,
        source_db: DatabaseConfig,
        target_db: DatabaseConfig,
        tables: Vec<OldTableConfig>,
    }

    // 尝试解析配置
    match serde_json::from_str::<Config>(&config_json) {
        Ok(config) => Ok(config),
        Err(_) => {
            // 尝试解析旧格式
            let old_config: OldConfig = serde_json::from_str(&config_json)
                .map_err(|e| format!("解析配置失败: {}", e))?;
            
            // 转换为新格式
            Ok(Config {
                source_ssh: old_config.source_ssh,
                target_ssh: old_config.target_ssh,
                source_db: old_config.source_db,
                target_db: old_config.target_db,
                tables: old_config.tables.into_iter().map(|t| TableConfig {
                    name: t.name,
                    columns: t.columns,
                    mask_rules: t.mask_rules,
                    structure_only: false,
                    ignore_foreign_keys: false,
                }).collect(),
            })
        }
    }
} 