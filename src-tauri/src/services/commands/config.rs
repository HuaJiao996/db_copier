// 配置相关命令

use crate::database::{Config, DatabaseConfig};
use crate::services::Storage;
use crate::services::commands::types::ConfigSummary;
use std::sync::Arc;
use tauri::State;
use log::{info, error};
use std::fs;

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