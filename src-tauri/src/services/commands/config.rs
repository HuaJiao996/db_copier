// 配置相关命令

use crate::database::{ColumnConfig, Config, DatabaseConfig, TableInfoChangeStatus};
use crate::db::{DbClient, TableConfig};
use crate::services::Storage;
use crate::services::commands::types::ConfigSummary;
use std::sync::Arc;
use tauri::State;
use log::{info, error, debug};
use std::fs;
use std::collections::{HashMap, HashSet};

/// 保存配置
#[tauri::command]
pub async fn save_config(
    config: Config,
    storage: State<'_, Arc<Storage>>,
) -> Result<(), String> {
    info!("保存配置: {}", config.name);
    info!("源数据库: {}:{}/{}, 目标数据库: {}:{}/{}", 
        config.source_db.host, config.source_db.port, config.source_db.database,
        config.target_db.host, config.target_db.port, config.target_db.database);
    info!("表数量: {}", config.tables.len());
    
    storage.save_config(&config)
        .await
        .map_err(|e| {
            error!("保存配置失败: {}", e);
            format!("保存配置失败: {}", e)
        })
}

/// 加载配置
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

/// 列出所有配置
#[tauri::command]
pub async fn list_configs(
    storage: State<'_, Arc<Storage>>,
) -> Result<Vec<String>, String> {
    storage.list_configs()
        .await
        .map_err(|e| format!("获取配置列表失败: {}", e))
}

/// 删除配置
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

/// 导入配置
#[tauri::command]
pub async fn import_config(
    file_path: String,
    storage: State<'_, Arc<Storage>>,
) -> Result<(), String> {
    // 读取JSON文件
    let config_json = fs::read_to_string(file_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 尝试解析配置
    match serde_json::from_str::<Config>(&config_json) {
        Ok(config) => {
            storage.save_config(&config).await.map_err(|e| format!("保存配置失败: {}", e))?;
            Ok(())
        }
        Err(_) => {
            error!("解析配置文件失败: {}", config_json);
            Err("解析配置文件失败".to_string())
        }
    }
}

/// 合并列配置
async fn merge_columns(
    client: &DbClient,
    table_name: &str,
    existing_columns: &[ColumnConfig],
) -> Result<Vec<ColumnConfig>, String> {
    debug!("Merging columns for table: {}", table_name);
    
    // 获取数据库中的列并转换为 HashSet
    let current_columns: HashSet<_> = client.get_table_columns(table_name)
        .await
        .map_err(|e| {
            error!("Failed to get columns for table {}: {}", table_name, e);
            e.to_string()
        })?
        .into_iter()
        .collect();
    
    // 创建现有列的映射，用于快速查找
    let existing_columns_map: HashMap<_, _> = existing_columns
        .iter()
        .map(|c| (&c.name, c))
        .collect();
    
    let mut merged_columns = Vec::new();
    
    // 处理当前存在的列
    for column_name in &current_columns {
        if let Some(&existing) = existing_columns_map.get(column_name) {
            debug!("Keeping existing column configuration: {}", column_name);
            let mut column_config = existing.clone();
            column_config.status = None;
            merged_columns.push(column_config);
        } else {
            debug!("Adding new column: {}", column_name);
            merged_columns.push(ColumnConfig {
                name: column_name.clone(),
                mask_rule: None,
                ignore: false,
                status: Some(TableInfoChangeStatus::Added),
            });
        }
    }
    
    // 处理已删除的列
    for column in existing_columns {
        if !current_columns.contains(&column.name) {
            debug!("Marking column as removed: {}", column.name);
            let mut removed_column = column.clone();
            removed_column.status = Some(TableInfoChangeStatus::Removed);
            merged_columns.push(removed_column);
        }
    }
    
    Ok(merged_columns)
}

/// 合并表配置
#[tauri::command]
pub async fn merge_table_config(
    database_config: DatabaseConfig,
    table_configs: Vec<TableConfig>,
) -> Result<Vec<TableConfig>, String> {
    info!("Starting table configuration merge");
    
    // 创建数据库客户端
    let client = DbClient::new(&database_config)
        .await
        .map_err(|e| {
            error!("Failed to create database client: {}", e);
            e.to_string()
        })?;
    
    // 获取当前数据库中的表
    let current_tables = client.get_tables()
        .await
        .map_err(|e| {
            error!("Failed to get table list: {}", e);
            e.to_string()
        })?;
    
    // 创建现有表配置的映射，用于快速查找
    let table_configs_map: HashMap<_, _> = table_configs
        .iter()
        .map(|t| (&t.name, t))
        .collect();
    
    let mut merged_tables = Vec::new();
    
    // 处理当前存在的表
    for table_name in &current_tables {
        debug!("Processing table: {}", table_name);
        
        let table_config = if let Some(&existing) = table_configs_map.get(table_name) {
            debug!("Found existing configuration for table: {}", table_name);
            let mut table_config = existing.clone();
            table_config.status = None;
            table_config
        } else {
            debug!("Creating new configuration for table: {}", table_name);
            TableConfig {
                name: table_name.clone(),
                columns: Vec::new(),
                structure_only: false,
                ignore_foreign_keys: false,
                ignore: true,
                status: Some(TableInfoChangeStatus::Added),
            }
        };
        
        // 合并列配置
        let merged_columns = merge_columns(&client, table_name, &table_config.columns).await?;
        
        let mut merged_table = table_config;
        merged_table.columns = merged_columns;
        merged_tables.push(merged_table);
    }
    
    // 处理已删除的表
    for table_config in table_configs.iter() {
        if !current_tables.contains(&table_config.name) {
            debug!("Marking table as removed: {}", table_config.name);
            let mut removed_table = table_config.clone();
            removed_table.status = Some(TableInfoChangeStatus::Removed);
            merged_tables.push(removed_table);
        }
    }
    
    info!("Table configuration merge completed. Total tables: {}", merged_tables.len());
    Ok(merged_tables)
}

/// 获取配置摘要
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
        has_source_ssh: config.source_db.ssh_config.is_some(),
        has_target_ssh: config.target_db.ssh_config.is_some(),
    };

    info!("配置摘要: 源库 {}, 目标库 {}, {} 个表, {} 个列", 
        summary.source_db, 
        summary.target_db,
        summary.table_count,
        summary.total_columns
    );

    Ok(summary)
}
