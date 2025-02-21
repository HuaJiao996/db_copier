use crate::models::Config;
use rusqlite::{params};
use std::path::PathBuf;
use tokio_rusqlite::Connection as AsyncConnection;
use serde_json;
use log::{info, error};
use std::convert::From;

#[derive(Debug)]
pub struct Storage {
    conn: AsyncConnection,
}

impl Storage {
    pub async fn new() -> Result<Self, tokio_rusqlite::Error> {
        // 确保数据目录存在
        let data_dir = PathBuf::from("data");
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            tokio_rusqlite::Error::Rusqlite(rusqlite::Error::InvalidPath(
                PathBuf::from(format!("创建数据目录失败: {}", e))
            ))
        })?;

        let db_path = data_dir.join("configs.db");
        info!("初始化数据库: {:?}", db_path);
        
        let conn = AsyncConnection::open(db_path).await.map_err(|e| {
            tokio_rusqlite::Error::Rusqlite(rusqlite::Error::InvalidPath(
                PathBuf::from(format!("打开数据库失败: {}", e))
            ))
        })?;

        // 初始化数据库表
        conn.call(|conn| {
            Ok(conn.execute(
                "CREATE TABLE IF NOT EXISTS configs (
                    name TEXT PRIMARY KEY,
                    content TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?)
        }).await?;

        info!("数据库初始化完成");
        Ok(Self { conn })
    }

    pub async fn save_config(&self, name: &str, config: &Config) -> Result<(), tokio_rusqlite::Error> {
        let content = serde_json::to_string(config).map_err(|e| {
            tokio_rusqlite::Error::Rusqlite(rusqlite::Error::InvalidParameterName(
                format!("序列化配置失败: {}", e)
            ))
        })?;

        let name_clone = name.to_string();
        let content_clone = content.clone();

        self.conn.call(move |conn| {
            Ok(conn.execute(
                "INSERT OR REPLACE INTO configs (name, content, updated_at) 
                 VALUES (?1, ?2, CURRENT_TIMESTAMP)",
                params![name_clone, content_clone],
            )?)
        }).await?;

        info!("保存配置成功: {}", name);
        Ok(())
    }

    pub async fn load_config(&self, name: &str) -> Result<Option<Config>, tokio_rusqlite::Error> {
        let name_clone = name.to_string();
        let result = self.conn.call(move |conn| {
            Ok(conn.query_row(
                "SELECT content FROM configs WHERE name = ?1",
                params![name_clone],
                |row| row.get::<_, String>(0),
            ))
        }).await;

        match result {
            Ok(Ok(content)) => {
                let config = serde_json::from_str(&content).map_err(|e| {
                    tokio_rusqlite::Error::Rusqlite(rusqlite::Error::InvalidParameterName(
                        format!("解析配置失败: {}", e)
                    ))
                })?;
                info!("加载配置成功: {}", name);
                Ok(Some(config))
            }
            Ok(Err(rusqlite::Error::QueryReturnedNoRows)) => {
                info!("配置不存在: {}", name);
                Ok(None)
            }
            Ok(Err(e)) => {
                error!("加载配置失败: {}", e);
                Err(tokio_rusqlite::Error::Rusqlite(e))
            }
            Err(e) => {
                error!("加载配置失败: {}", e);
                Err(e)
            }
        }
    }

    pub async fn list_configs(&self) -> Result<Vec<String>, tokio_rusqlite::Error> {
        self.conn.call(|conn| {
            let mut stmt = conn.prepare("SELECT name FROM configs ORDER BY name")?;
            let names = stmt.query_map([], |row| row.get(0))?
                .collect::<Result<Vec<String>, _>>()?;
            Ok(names)
        }).await
    }

    pub async fn delete_config(&self, name: &str) -> Result<bool, tokio_rusqlite::Error> {
        let name_clone = name.to_string();
        let rows = self.conn.call(move |conn| {
            Ok(conn.execute("DELETE FROM configs WHERE name = ?1", params![name_clone])?)
        }).await?;
        
        if rows > 0 {
            info!("删除配置成功: {}", name);
        } else {
            info!("要删除的配置不存在: {}", name);
        }
        
        Ok(rows > 0)
    }
} 