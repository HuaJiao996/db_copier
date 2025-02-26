// 数据库连接相关命令

use crate::{database::{Config, DbCopier, TableInfo}, db::DatabaseConfig};
use log::{info, error};

/// 测试数据库连接
#[tauri::command]
pub async fn test_connection(config: DatabaseConfig) -> Result<String, String> {
    // 记录详细的连接信息
    info!("测试数据库连接请求:");
    
    // 检查是否测试
    let test = !config.host.is_empty() && 
                      !config.database.is_empty() && 
                      !config.username.is_empty();
    
    // 记录测试信息
    if test {
        info!("测试数据库: {}:{}/{}, 用户: {}, SSL: {}", 
            config.host, config.port, config.database,
            config.username, config.ssl_mode);
        
        if config.ssh_config.is_some() {
            info!("使用SSH隧道连接源数据库");
        }
    }
    
    
    // 如果两个数据库都没有配置，返回错误
    if !test{
        error!("数据库配置无效");
        return Err("请提供有效的数据库配置".to_string());
    }
    
    // 测试源数据库连接
    match DbCopier::test_connection(&config).await {
        Ok(_) => {
            info!("数据库连接测试成功");
            return Ok("数据库连接成功".to_string());
        },
        Err(e) => {
            error!("数据库连接测试失败: {}", e);
            return Err(format!("数据库连接失败: {}", e));
        }
    }
    
}

/// 获取表列表
#[tauri::command]
pub async fn get_tables(config: Config) -> Result<Vec<String>, String> {
    info!("开始获取表列表");
    let copier = DbCopier::new(&config)
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

/// 获取表的列信息
#[tauri::command]
pub async fn get_table_columns(
    config: Config,
    table_name: String
) -> Result<Vec<String>, String> {
    info!("开始获取表 {} 的列信息", table_name);
    let copier = DbCopier::new(&config)
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

/// 获取表的详细信息
#[tauri::command]
pub async fn get_table_info(
    config: Config,
    table_name: String,
) -> Result<TableInfo, String> {
    info!("开始获取表 {} 的详细信息", table_name);
    let copier = DbCopier::new(&config)
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

/// 同步表结构
#[tauri::command]
pub async fn sync_table_structure(
    config: Config,
    table_name: String,
) -> Result<Vec<String>, String> {
    info!("开始同步表 {} 的结构", table_name);
    let copier = DbCopier::new(&config)
        .await
        .map_err(|e| {
            error!("创建数据库连接失败: {}", e);
            e.to_string()
        })?;
    
    info!("数据库连接成功，正在获取最新列信息");
    let columns = copier.get_table_columns(&table_name)
        .await
        .map_err(|e| {
            error!("获取列信息失败: {}", e);
            e.to_string()
        })?;
    
    info!("成功同步表 {} 的结构，共 {} 列", table_name, columns.len());
    Ok(columns)
}

/// 比较表结构差异
#[tauri::command]
pub async fn compare_table_structure(
    current_columns: Vec<String>,
    saved_columns: Vec<String>
) -> Result<serde_json::Value, String> {
    info!("比较表结构差异");
    
    let added = current_columns.iter()
        .filter(|col| !saved_columns.contains(col))
        .cloned()
        .collect::<Vec<String>>();
    
    let removed = saved_columns.iter()
        .filter(|col| !current_columns.contains(col))
        .cloned()
        .collect::<Vec<String>>();
    
    let has_changes = !added.is_empty() || !removed.is_empty();
    
    let result = serde_json::json!({
        "added": added,
        "removed": removed,
        "hasChanges": has_changes
    });
    
    Ok(result)
} 