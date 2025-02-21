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
use serde::Serialize;

#[derive(Clone)]
pub struct TaskStore(pub Arc<Mutex<HashMap<String, TaskStatus>>>);

#[derive(Debug, Serialize)]
pub struct ConfigSummary {
    pub source_db: String,
    pub target_db: String,
    pub table_count: usize,
    pub total_columns: usize,
    pub has_source_ssh: bool,
    pub has_target_ssh: bool,
}

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
    storage: State<'_, Arc<Storage>>,
) -> Result<String, String> {
    let task_id = Local::now().format("%Y%m%d%H%M%S").to_string();
    info!("Starting copy task: {}", task_id);
    
    let task_status = TaskStatus {
        id: task_id.clone(),
        status: TaskState::Running,
        start_time: Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        end_time: None,
        message: None,
        progress: Some(Progress {
            current: 0,
            total: config.tables.len(),
            table_name: String::new(),
        }),
    };

    // 保存初始任务状态
    {
        let mut tasks = task_store.0.lock().unwrap();
        tasks.insert(task_id.clone(), task_status.clone());
    }
    storage.save_task(&task_status)
        .await
        .map_err(|e| format!("保存任务状态失败: {}", e))?;

    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let storage_clone = storage.inner().clone();
    let task_store_clone = task_store.0.clone();
    let task_id_clone = task_id.clone();
    
    // 在后台执行复制任务
    tokio::spawn(async move {
        async fn update_task(
            task_id: &str, 
            task_store: &Arc<Mutex<HashMap<String, TaskStatus>>>,
            storage: &Arc<Storage>,
            update_fn: impl FnOnce(&mut TaskStatus)
        ) -> Result<(), String> {
            let mut task = {
                let mut tasks = task_store.lock().unwrap();
                tasks.get_mut(task_id)
                    .ok_or_else(|| "任务不存在".to_string())?
                    .clone()
            };
            
            update_fn(&mut task);
            
            // 更新内存中的任务状态
            {
                let mut tasks = task_store.lock().unwrap();
                if let Some(t) = tasks.get_mut(task_id) {
                    *t = task.clone();
                }
            }
            
            // 保存到数据库
            storage.save_task(&task)
                .await
                .map_err(|e| format!("保存任务状态失败: {}", e))?;
            
            Ok(())
        }

        match DbCopier::new(&config, memory_monitor).await {
            Ok(copier) => {
                for (i, table) in config.tables.iter().enumerate() {
                    info!("Copying table {}/{}: {}", i + 1, config.tables.len(), table.name);
                    
                    // 更新进度
                    let _ = update_task(
                        &task_id_clone,
                        &task_store_clone,
                        &storage_clone,
                        |task| {
                            if let Some(progress) = &mut task.progress {
                                progress.current = i;
                                progress.table_name = table.name.clone();
                            }
                        }
                    ).await;

                    match copier.copy_table(table).await {
                        Ok(_) => {
                            info!("Successfully copied table: {}", table.name);
                        }
                        Err(e) => {
                            error!("Failed to copy table {}: {}", table.name, e);
                            let _ = update_task(
                                &task_id_clone,
                                &task_store_clone,
                                &storage_clone,
                                |task| {
                                    task.status = TaskState::Failed;
                                    task.end_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                                    task.message = Some(format!("复制表 {} 失败: {}", table.name, e));
                                }
                            ).await;
                            return;
                        }
                    }
                }

                // 更新完成状态
                let _ = update_task(
                    &task_id_clone,
                    &task_store_clone,
                    &storage_clone,
                    |task| {
                        task.status = TaskState::Completed;
                        task.end_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        task.message = Some("复制完成".to_string());
                        if let Some(progress) = &mut task.progress {
                            progress.current = config.tables.len();
                        }
                    }
                ).await;
                info!("Copy task completed: {}", task_id_clone);
            }
            Err(e) => {
                error!("Failed to initialize copier: {}", e);
                let _ = update_task(
                    &task_id_clone,
                    &task_store_clone,
                    &storage_clone,
                    |task| {
                        task.status = TaskState::Failed;
                        task.end_time = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        task.message = Some(format!("初始化失败: {}", e));
                    }
                ).await;
            }
        }
    });

    Ok(task_id)
}

#[tauri::command]
pub async fn get_task_status(
    task_id: String,
    task_store: State<'_, TaskStore>,
    storage: State<'_, Arc<Storage>>,
) -> Result<TaskStatus, String> {
    // 先从内存中获取
    {
        let tasks = task_store.0.lock().unwrap();
        if let Some(task) = tasks.get(&task_id) {
            return Ok(task.clone());
        }
    }

    // 如果内存中没有，从数据库获取
    match storage.get_task(&task_id).await {
        Ok(Some(task)) => Ok(task),
        Ok(None) => Err("任务不存在".to_string()),
        Err(e) => Err(format!("获取任务状态失败: {}", e)),
    }
}

#[tauri::command]
pub async fn get_table_columns(
    config: Config,
    table_name: String
) -> Result<Vec<String>, String> {
    info!("开始获取表 {} 的列信息", table_name);
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| {
            error!("创建数据库连接失败: {}", e);
            e.to_string()
        })?;
    
    info!("数据库连接成功，正在获取列信息");
    let columns = copier.get_table_columns(&table_name)
        .await
        .map_err(|e| {
            error!("获取列信息失败: {}", e);
            e.to_string()
        })?;
    
    info!("成功获取到表 {} 的 {} 个列", table_name, columns.len());
    Ok(columns)
}

#[tauri::command]
pub async fn get_tables(config: Config) -> Result<Vec<String>, String> {
    info!("开始获取表列表");
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| {
            error!("创建数据库连接失败: {}", e);
            e.to_string()
        })?;
    
    info!("数据库连接成功，正在获取表列表");
    let tables = copier.get_tables()
        .await
        .map_err(|e| {
            error!("获取表列表失败: {}", e);
            e.to_string()
        })?;
    
    info!("成功获取到 {} 个表", tables.len());
    Ok(tables)
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
    info!("开始获取表 {} 的详细信息", table_name);
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100));
    let copier = DbCopier::new(&config, memory_monitor)
        .await
        .map_err(|e| {
            error!("创建数据库连接失败: {}", e);
            e.to_string()
        })?;
    
    info!("数据库连接成功，正在获取表信息");
    let table_info = copier.get_table_info(&table_name)
        .await
        .map_err(|e| {
            error!("获取表信息失败: {}", e);
            e.to_string()
        })?;
    
    info!("成功获取到表 {} 的详细信息", table_name);
    Ok(table_info)
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
            
            // 转换为新格式，移除了 name 字段
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

#[tauri::command]
pub fn get_config_summary(config: Config) -> Result<ConfigSummary, String> {
    info!("生成配置摘要信息");
    
    // 计算所有表的列总数
    let total_columns = config.tables.iter()
        .map(|t| t.columns.len())
        .sum();

    // 格式化数据库连接信息
    let format_db_info = |db: &DatabaseConfig| -> String {
        format!("{}@{}:{}/{}", 
            db.username,
            db.host,
            db.port,
            db.database
        )
    };

    let summary = ConfigSummary {
        source_db: format_db_info(&config.source_db),
        target_db: format_db_info(&config.target_db),
        table_count: config.tables.len(),
        total_columns,
        has_source_ssh: config.source_ssh.is_some(),
        has_target_ssh: config.target_ssh.is_some(),
    };

    info!("配置摘要: 源库 {}, 目标库 {}, {} 个表, {} 个列", 
        summary.source_db, 
        summary.target_db,
        summary.table_count,
        summary.total_columns
    );

    Ok(summary)
}

#[tauri::command]
pub async fn get_all_tasks(
    task_store: State<'_, TaskStore>,
    storage: State<'_, Arc<Storage>>,
) -> Result<Vec<TaskStatus>, String> {
    // 从数据库获取所有任务
    let mut tasks = storage.get_all_tasks()
        .await
        .map_err(|e| format!("获取任务列表失败: {}", e))?;

    // 合并内存中的任务状态
    {
        let memory_tasks = task_store.0.lock().unwrap();
        for task in memory_tasks.values() {
            if let Some(existing_task) = tasks.iter_mut().find(|t| t.id == task.id) {
                *existing_task = task.clone();
            } else {
                tasks.push(task.clone());
            }
        }
    }

    // 按时间倒序排序
    tasks.sort_by(|a, b| {
        b.start_time.as_ref().unwrap_or(&"".to_string())
            .cmp(a.start_time.as_ref().unwrap_or(&"".to_string()))
    });

    Ok(tasks)
} 