// 数据库连接相关命令

use crate::{database::DbClient, db::DatabaseConfig};
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
    } else {
        error!("数据库配置无效");
        return Err("请提供有效的数据库配置".to_string());
    }
    
    // 测试源数据库连接
    match DbClient::new(&config).await {
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
pub async fn get_tables(config: DatabaseConfig) -> Result<Vec<String>, String> {
    info!("开始获取表列表");
    let client = DbClient::new(&config)
        .await
        .map_err(|e| {
            error!("创建数据库连接失败: {}", e);
            e.to_string()
        })?;
    
    info!("数据库连接成功，正在获取表列表");
    let tables = client.get_tables()
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
    config: DatabaseConfig,
    table_name: String
) -> Result<Vec<String>, String> {
    info!("开始获取表 {} 的列信息", table_name);
    let client = DbClient::new(&config)
        .await
        .map_err(|e| {
            error!("创建数据库连接失败: {}", e);
            e.to_string()
        })?;
    
    info!("数据库连接成功，正在获取列信息");
    let columns = client.get_table_columns(&table_name)
        .await
        .map_err(|e| {
            error!("获取列信息失败: {}", e);
            e.to_string()
        })?;
    
    info!("成功获取到表 {} 的 {} 个列", table_name, columns.len());
    Ok(columns)
}
