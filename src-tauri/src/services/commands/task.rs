// 任务相关命令

use crate::database::{Config, TaskStatus, TaskState, Progress};
use crate::db_copier::DbCopier;
use crate::services::Storage;
use crate::services::commands::types::TaskStore;
use std::sync::Arc;
use chrono::Local;
use tauri::State;
use log::{info, error};
use tokio;
use std::collections::HashMap;
use std::sync::Mutex;

/// 开始复制任务
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

        match DbCopier::new(&config).await {
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

/// 获取任务状态
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

/// 获取所有任务
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