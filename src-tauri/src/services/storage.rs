use crate::database::{Config, TaskStatus};
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

    pub async fn save_config(&self, config: &Config) -> Result<(), tokio_rusqlite::Error> {
        let name = config.name.clone();
        let content = serde_json::to_string(config).map_err(|e| {
            tokio_rusqlite::Error::Rusqlite(rusqlite::Error::InvalidParameterName(
                format!("序列化配置失败: {}", e)
            ))
        })?;

        self.conn.call(move |conn| {
            Ok(conn.execute(
                "INSERT OR REPLACE INTO configs (name, content, updated_at) 
                 VALUES (?1, ?2, CURRENT_TIMESTAMP)",
                params![name, content],
            )?)
        }).await?;

        info!("保存配置成功");
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

    // 保存任务状态
    pub async fn save_task(&self, task: &TaskStatus) -> Result<(), tokio_rusqlite::Error> {
        let task_json = serde_json::to_string(task)
            .map_err(|e| tokio_rusqlite::Error::Rusqlite(rusqlite::Error::InvalidParameterName(
                format!("序列化任务失败: {}", e)
            )))?;

        let task_id = task.id.clone();
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO tasks (id, content) VALUES (?1, ?2)",
                params![task_id, task_json],
            )?;
            Ok(())
        }).await?;

        Ok(())
    }

    // 获取所有任务
    pub async fn get_all_tasks(&self) -> Result<Vec<TaskStatus>, tokio_rusqlite::Error> {
        self.conn.call(|conn| {
            let mut stmt = conn.prepare("SELECT content FROM tasks ORDER BY id DESC")?;
            let tasks = stmt.query_map([], |row| {
                let content: String = row.get(0)?;
                serde_json::from_str(&content)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(
                        format!("解析任务失败: {}", e)
                    ))
            })?
            .collect::<Result<Vec<TaskStatus>, _>>()?;
            Ok(tasks)
        }).await
    }

    // 获取单个任务
    pub async fn get_task(&self, id: &str) -> Result<Option<TaskStatus>, tokio_rusqlite::Error> {
        let id_clone = id.to_string();
        self.conn.call(move |conn| {
            match conn.query_row(
                "SELECT content FROM tasks WHERE id = ?1",
                params![id_clone],
                |row| {
                    let content: String = row.get(0)?;
                    serde_json::from_str(&content)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(
                            format!("解析任务失败: {}", e)
                        ))
                },
            ) {
                Ok(task) => Ok(Some(task)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e))
            }
        }).await
    }

    // 初始化数据库时添加任务表
    pub async fn init_db(&self) -> Result<(), tokio_rusqlite::Error> {
        self.conn.call(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    content TEXT NOT NULL
                )",
                [],
            )?;
            Ok(())
        }).await?;
        Ok(())
    }
} 