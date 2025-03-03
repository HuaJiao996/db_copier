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

