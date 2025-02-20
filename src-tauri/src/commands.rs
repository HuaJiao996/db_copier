use crate::models::{Config, TaskStatus, TaskState, Progress};
use crate::db::{DbCopier, TableInfo};
use crate::monitor::MemoryMonitor;
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Local;
use tauri::State;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use log::{info, error};

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
pub async fn save_config(name: String, config: Config) -> Result<(), String> {
    let config_dir = PathBuf::from("configs");
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;
    
    let config_path = config_dir.join(format!("{}.json", name));
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(config_path, config_json)
        .map_err(|e| format!("保存配置失败: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn load_config(name: String) -> Result<Config, String> {
    let config_path = PathBuf::from("configs").join(format!("{}.json", name));
    let config_json = fs::read_to_string(config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    
    serde_json::from_str(&config_json)
        .map_err(|e| format!("解析配置失败: {}", e))
}

#[tauri::command]
pub async fn list_configs() -> Result<Vec<String>, String> {
    let config_dir = PathBuf::from("configs");
    if !config_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut configs = Vec::new();
    for entry in fs::read_dir(config_dir)
        .map_err(|e| format!("读取配置目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取配置文件失败: {}", e))?;
        if let Some(name) = entry.path()
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from) {
            configs.push(name);
        }
    }
    
    Ok(configs)
}

#[tauri::command]
pub async fn delete_config(name: String) -> Result<(), String> {
    let config_path = PathBuf::from("configs").join(format!("{}.json", name));
    fs::remove_file(config_path)
        .map_err(|e| format!("删除配置失败: {}", e))?;
    Ok(())
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
    
    copier.sync_table_structure(&table_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_memory_usage(memory_monitor: tauri::State<'_, Arc<MemoryMonitor>>) -> Result<usize, String> {
    Ok(memory_monitor.get_usage())
} 