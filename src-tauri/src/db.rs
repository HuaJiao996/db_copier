use crate::models::{Config, DatabaseConfig, SSHConfig, TableConfig, MaskRule, MaskRuleType};
use ssh2::Session;
use std::net::TcpStream;
use tokio_postgres::{Client, Config as PgConfig};
use tokio_postgres::Error as PgError;
use std::error::Error;
use std::fmt;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use futures::stream::{self, StreamExt};
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::pin::Pin;
use std::future::Future;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::monitor::MemoryMonitor;
use tokio_postgres::types::ToSql;
use native_tls;
use postgres_native_tls;
use log::{info, error};

#[derive(Debug)]
pub enum DbError {
    Connection(String),
    Query(String),
    SSH(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Connection(msg) => write!(f, "Connection error: {}", msg),
            DbError::Query(msg) => write!(f, "Query error: {}", msg),
            DbError::SSH(msg) => write!(f, "SSH error: {}", msg),
        }
    }
}

impl Error for DbError {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnInfo {
    name: String,
    data_type: String,
    is_nullable: bool,
    column_default: Option<String>,
    character_maximum_length: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableInfo {
    name: String,
    columns: Vec<ColumnInfo>,
    indexes: Vec<String>,
    constraints: Vec<String>,
}

pub struct DbCopier {
    source_client: Client,
    target_client: Client,
    _source_ssh_session: Option<Session>,
    _target_ssh_session: Option<Session>,
    table_info_cache: Arc<RwLock<HashMap<String, TableInfo>>>,
    memory_monitor: Arc<MemoryMonitor>,
}

impl DbCopier {
    pub async fn new(config: &Config, memory_monitor: Arc<MemoryMonitor>) -> Result<Self, DbError> {
        // 连接源数据库
        let (source_client, source_ssh_session) = match &config.source_ssh {
            Some(ssh_config) => {
                let (client, session) = Self::connect_with_ssh(
                    ssh_config,
                    &config.source_db,
                ).await?;
                (client, Some(session))
            }
            None => {
                let client = Self::connect_db(&config.source_db).await?;
                (client, None)
            }
        };

        // 连接目标数据库
        let (target_client, target_ssh_session) = match &config.target_ssh {
            Some(ssh_config) => {
                let (client, session) = Self::connect_with_ssh(
                    ssh_config,
                    &config.target_db,
                ).await?;
                (client, Some(session))
            }
            None => {
                let client = Self::connect_db(&config.target_db).await?;
                (client, None)
            }
        };

        Ok(Self {
            source_client,
            target_client,
            _source_ssh_session: source_ssh_session,
            _target_ssh_session: target_ssh_session,
            table_info_cache: Arc::new(RwLock::new(HashMap::new())),
            memory_monitor,
        })
    }

    async fn connect_with_ssh(
        ssh_config: &SSHConfig,
        db_config: &DatabaseConfig,
    ) -> Result<(Client, Session), DbError> {
        info!("开始建立 SSH 连接 {}:{}", ssh_config.host, ssh_config.port);
        
        let tcp = TcpStream::connect(format!("{}:{}", ssh_config.host, ssh_config.port))
            .map_err(|e| {
                error!("SSH TCP 连接失败: {}", e);
                DbError::SSH(e.to_string())
            })?;

        info!("创建 SSH 会话");
        let mut session = Session::new()
            .map_err(|e| {
                error!("创建 SSH 会话失败: {}", e);
                DbError::SSH(e.to_string())
            })?;
        session.set_tcp_stream(tcp);
        
        info!("进行 SSH 握手");
        session.handshake()
            .map_err(|e| {
                error!("SSH 握手失败: {}", e);
                DbError::SSH(e.to_string())
            })?;

        // SSH认证
        info!("开始 SSH 认证");
        if let Some(key_path) = &ssh_config.private_key_path {
            info!("使用密钥认证: {}", key_path);
            session.userauth_pubkey_file(
                &ssh_config.username,
                None,
                std::path::Path::new(key_path),
                None,
            ).map_err(|e| {
                error!("SSH 密钥认证失败: {}", e);
                DbError::SSH(e.to_string())
            })?;
        } else if let Some(password) = &ssh_config.password {
            info!("使用密码认证");
            session.userauth_password(&ssh_config.username, password)
                .map_err(|e| {
                    error!("SSH 密码认证失败: {}", e);
                    DbError::SSH(e.to_string())
                })?;
        }

        info!("设置 SSH 端口转发");
        let local_port = Self::find_available_port()?;
        info!("使用本地端口: {}", local_port);
        let _listener = session.channel_forward_listen(local_port, None, None)
            .map_err(|e| {
                error!("设置端口转发失败: {}", e);
                DbError::SSH(e.to_string())
            })?;

        info!("通过 SSH 隧道连接数据库");
        let mut db_config = db_config.clone();
        db_config.host = "127.0.0.1".to_string();
        db_config.port = local_port;
        let client = Self::connect_db(&db_config).await?;

        info!("SSH 隧道和数据库连接都已建立成功");
        Ok((client, session))
    }

    fn find_available_port() -> Result<u16, DbError> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .map_err(|e| DbError::Connection(e.to_string()))?;
        let port = listener.local_addr()
            .map_err(|e| DbError::Connection(e.to_string()))?
            .port();
        Ok(port)
    }

    async fn connect_db(config: &DatabaseConfig) -> Result<Client, DbError> {
        info!("开始连接数据库 {}:{}/{}", config.host, config.port, config.database);
        info!("SSL 模式: {}", config.ssl_mode);
        
        let mut pg_config = PgConfig::new();
        pg_config
            .host(&config.host)
            .port(config.port)
            .dbname(&config.database)
            .user(&config.username)
            .password(&config.password)
            .application_name("db_copier")
            .connect_timeout(Duration::from_secs(10))
            .keepalives(true)
            .keepalives_idle(Duration::from_secs(30))
            .ssl_mode(match config.ssl_mode.as_str() {
                "require" => {
                    info!("使用 SSL 加密连接");
                    tokio_postgres::config::SslMode::Require
                },
                "disable" => {
                    info!("禁用 SSL 连接");
                    tokio_postgres::config::SslMode::Disable
                },
                _ => {
                    info!("使用首选 SSL 连接");
                    tokio_postgres::config::SslMode::Prefer
                },
            });

        let mut attempts = 0;
        let max_attempts = 3;
        let mut last_error = None;

        while attempts < max_attempts {
            info!("尝试连接数据库 (第 {} 次尝试)", attempts + 1);
            
            match async {
                info!("创建 TLS 连接器");
                let mut builder = native_tls::TlsConnector::builder();
                
                // 如果启用了 SSL，允许自签名证书
                if config.ssl_mode == "require" {
                    info!("允许自签名证书");
                    builder.danger_accept_invalid_certs(true);
                }
                
                let connector = builder
                    .build()
                    .map_err(|e| DbError::Connection(format!("创建 TLS 连接器失败: {}", e)))?;
                    
                let connector = postgres_native_tls::MakeTlsConnector::new(connector);

                info!("建立数据库连接");
                let (client, connection) = pg_config
                    .connect(connector)
                    .await
                    .map_err(|e| DbError::Connection(format!("连接数据库失败 ({}:{}): {}", config.host, config.port, e)))?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("数据库连接错误: {}", e);
                    }
                });

                info!("数据库连接成功");
                Ok(client)
            }.await {
                Ok(client) => {
                    info!("成功建立数据库连接");
                    return Ok(client)
                },
                Err(e) => {
                    error!("连接失败: {}", e);
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < max_attempts {
                        let wait_time = 2u64.pow(attempts as u32);
                        info!("等待 {} 秒后重试", wait_time);
                        tokio::time::sleep(Duration::from_secs(wait_time)).await;
                    }
                }
            }
        }

        error!("达到最大重试次数，连接失败");
        Err(last_error.unwrap())
    }

    #[allow(dead_code)]
    fn apply_mask_rule(&self, value: &str, rule: &MaskRule) -> String {
        match rule.rule_type {
            MaskRuleType::Hash => {
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                format!("{:x}", hasher.finalize())
            },
            MaskRuleType::Fixed => {
                rule.pattern.clone().unwrap_or_else(|| "****".to_string())
            },
            MaskRuleType::Pattern => {
                if let Some(pattern) = &rule.pattern {
                    let mut result = String::with_capacity(value.len());
                    let mut chars = value.chars();
                    for p in pattern.chars() {
                        match p {
                            '#' => {
                                if let Some(c) = chars.next() {
                                    result.push(c);
                                }
                            },
                            '*' => {
                                if chars.next().is_some() {
                                    result.push('*');
                                }
                            },
                            _ => result.push(p),
                        }
                    }
                    result
                } else {
                    value.to_string()
                }
            }
        }
    }

    #[allow(dead_code)]
    async fn execute_with_retry<F, T, E>(&self, f: F, retries: u32) -> Result<T, E>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < retries {
                        sleep(Duration::from_secs(2u64.pow(attempts))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    // 添加内存监控使用
    fn update_memory_usage(&self, delta: isize) {
        self.memory_monitor.update_usage(delta);
    }

    async fn execute_batch_insert(
        &self,
        table: &TableConfig,
        batch_values: &mut Vec<String>,
        batch_params: &mut Vec<String>,
        memory_counter: &AtomicUsize,
    ) -> Result<(), DbError> {
        if batch_values.is_empty() {
            return Ok(());
        }

        let total_size = batch_values.iter().map(|s| s.len()).sum::<usize>();
        self.update_memory_usage(-(total_size as isize));
        memory_counter.fetch_sub(total_size, Ordering::Relaxed);

        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table.name,
            table.columns.join(", "),
            batch_values.join(", ")
        );

        self.target_client
            .execute(&insert_sql, &batch_params.iter().map(|s| s as _).collect::<Vec<_>>())
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        batch_values.clear();
        batch_params.clear();

        Ok(())
    }

    async fn copy_table_data_stream(&self, table: &TableConfig) -> Result<(), DbError> {
        const BATCH_SIZE: usize = 1000;
        let mut values: Vec<String> = Vec::new();
        let mut batch_params: Vec<String> = Vec::new();
        let mut value_count = 0;

        // 构建查询语句
        let columns = table.columns.join(", ");
        let select_sql = format!("SELECT {} FROM {}", columns, table.name);

        // 执行查询
        let source_rows = self.source_client
            .query(&select_sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // 构建插入语句
        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES",
            table.name,
            columns
        );

        let client = &self.target_client;
        
        // 处理每一行数据
        for row in source_rows {
            let mut row_placeholders = Vec::new();
            for column in table.columns.iter() {
                value_count += 1;
                let value: String = row.get(column.as_str());
                // 更新内存使用统计
                self.update_memory_usage(value.len() as isize);
                values.push(value);
                row_placeholders.push(format!("${}", value_count));
            }
            batch_params.push(format!("({})", row_placeholders.join(", ")));

            if batch_params.len() >= BATCH_SIZE {
                let full_insert_sql = format!("{} {}", insert_sql, batch_params.join(", "));
                let params: Vec<&(dyn ToSql + Sync)> = values.iter().map(|v| v as &(dyn ToSql + Sync)).collect();
                client.execute(&full_insert_sql, &params[..]).await
                    .map_err(|e| DbError::Query(e.to_string()))?;
                
                // 更新内存使用统计（减少）
                let freed_memory: isize = -(values.iter().map(|s| s.len() as isize).sum::<isize>());
                self.update_memory_usage(freed_memory);
                
                values.clear();
                batch_params.clear();
                value_count = 0;
            }
        }

        // 处理剩余的数据
        if !batch_params.is_empty() {
            let full_insert_sql = format!("{} {}", insert_sql, batch_params.join(", "));
            let params: Vec<&(dyn ToSql + Sync)> = values.iter().map(|v| v as &(dyn ToSql + Sync)).collect();
            client.execute(&full_insert_sql, &params[..]).await
                .map_err(|e| DbError::Query(e.to_string()))?;
            
            // 更新内存使用统计（减少）
            let freed_memory: isize = -(values.iter().map(|s| s.len() as isize).sum::<isize>());
            self.update_memory_usage(freed_memory);
        }

        Ok(())
    }

    pub async fn copy_table(&self, table: &TableConfig) -> Result<(), DbError> {
        // 获取并同步表结构
        let table_info = self.get_table_info(&table.name).await?;
        self.sync_table_structure(&table_info, table.ignore_foreign_keys).await?;

        // 如果只复制结构，则直接返回
        if table.structure_only {
            return Ok(());
        }

        // 使用流式处理复制数据
        let columns = table.columns.join("\", \"");
        let select_sql = format!("SELECT \"{}\" FROM \"{}\"", columns, table.name);

        // 执行查询并插入数据
        let rows = self.source_client
            .query(&select_sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        if !rows.is_empty() {
            let insert_sql = format!(
                "INSERT INTO \"{}\" (\"{}\") VALUES",
                table.name,
                columns
            );

            // 构建批量插入的值
            let mut values = Vec::new();
            let mut params = Vec::new();
            let mut param_count = 1;

            for row in rows {
                let mut row_values = Vec::new();
                for column in &table.columns {
                    let value: String = row.get(column.as_str());
                    params.push(value);
                    row_values.push(format!("${}", param_count));
                    param_count += 1;
                }
                values.push(format!("({})", row_values.join(", ")));
            }

            // 执行批量插入
            let full_insert_sql = format!("{} {}", insert_sql, values.join(", "));
            self.target_client
                .execute(&full_insert_sql, &params.iter().map(|s| s as &(dyn ToSql + Sync)).collect::<Vec<_>>())
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    async fn get_create_table_sql(&self, table: &TableConfig) -> Result<String, DbError> {
        let sql = format!(
            "SELECT pg_get_createtable_command('{}') as create_sql",
            table.name
        );

        let row = self.source_client
            .query_one(&sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let create_sql: String = row.get("create_sql");
        Ok(create_sql)
    }

    pub async fn get_tables(&self) -> Result<Vec<String>, DbError> {
        let rows = self.source_client
            .query(
                "SELECT table_name FROM information_schema.tables 
                 WHERE table_schema = 'public'
                 ORDER BY table_name",
                &[],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<String>, DbError> {
        let rows = self.source_client
            .query(
                "SELECT column_name FROM information_schema.columns 
                 WHERE table_schema = 'public' AND table_name = $1
                 ORDER BY ordinal_position",
                &[&table_name],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, DbError> {
        // 先检查缓存
        if let Some(info) = self.table_info_cache.read().await.get(table_name) {
            return Ok(info.clone());
        }

        // 如果缓存中没有，则从数据库获取
        let rows = self.source_client
            .query(
                "SELECT column_name, data_type, is_nullable, column_default, character_maximum_length
                 FROM information_schema.columns 
                 WHERE table_schema = 'public' AND table_name = $1
                 ORDER BY ordinal_position",
                &[&table_name],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut columns = Vec::new();
        for row in rows {
            let is_nullable: String = row.get("is_nullable");
            columns.push(ColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: is_nullable.eq_ignore_ascii_case("YES"),
                column_default: row.get("column_default"),
                character_maximum_length: row.get("character_maximum_length"),
            });
        }

        // 获取索引信息
        let indexes = self.get_table_indexes(table_name).await?;

        // 获取约束信息
        let constraints = self.get_table_constraints(table_name).await?;

        let info = TableInfo {
            name: table_name.to_string(),
            columns,
            indexes,
            constraints,
        };

        // 更新缓存
        self.table_info_cache.write().await.insert(table_name.to_string(), info.clone());

        Ok(info)
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<String>, DbError> {
        let rows = self.source_client
            .query(
                "SELECT indexdef FROM pg_indexes WHERE tablename = $1",
                &[&table_name],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<String>, DbError> {
        let rows = self.source_client
            .query(
                "SELECT pg_get_constraintdef(c.oid) as constraint_def
                 FROM pg_constraint c
                 JOIN pg_namespace n ON n.oid = c.connamespace
                 WHERE conrelid = (SELECT oid FROM pg_class WHERE relname = $1 AND relnamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public'))
                 AND n.nspname = 'public'",
                &[&table_name],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn sync_table_structure(&self, table_info: &TableInfo, ignore_foreign_keys: bool) -> Result<(), DbError> {
        // 删除目标表(如果存在)
        self.target_client
            .execute(
                &format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table_info.name),
                &[],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // 创建表
        let mut create_table = format!("CREATE TABLE \"{}\" (\n", table_info.name);
        
        // 添加列定义
        let column_defs: Vec<String> = table_info.columns
            .iter()
            .map(|col| {
                let mut def = format!("    \"{}\" {}", col.name, col.data_type);
                if !col.is_nullable {
                    def.push_str(" NOT NULL");
                }
                if let Some(default) = &col.column_default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
                def
            })
            .collect();
        
        create_table.push_str(&column_defs.join(",\n"));
        create_table.push_str("\n)");

        self.target_client
            .execute(&create_table, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // 添加约束（如果不忽略外键）
        if !ignore_foreign_keys {
            for constraint in &table_info.constraints {
                // 跳过外键约束
                if !constraint.contains("FOREIGN KEY") {
                    self.target_client
                        .execute(
                            &format!("ALTER TABLE \"{}\" ADD {}", table_info.name, constraint),
                            &[],
                        )
                        .await
                        .map_err(|e| DbError::Query(e.to_string()))?;
                }
            }
        }

        // 添加索引
        for index in &table_info.indexes {
            self.target_client
                .execute(index, &[])
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn copy_tables(&self, tables: &[TableConfig]) -> Result<(), DbError> {
        // 限制并发数
        let semaphore = Arc::new(Semaphore::new(4));
        
        let results = stream::iter(tables)
            .map(|table| {
                let sem = semaphore.clone();
                let table = table.clone();
                async move {
                    let _permit = sem.acquire().await.unwrap();
                    self.copy_table(&table).await
                }
            })
            .buffer_unordered(4) // 最多4个并发任务
            .collect::<Vec<_>>()
            .await;

        // 检查是否有错误
        for result in results {
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub async fn copy_table_data(
    source_client: &Client,
    target_client: &Client, 
    table_name: &str,
    columns: &[String]
) -> Result<(), Box<dyn std::error::Error>> {
    let select_sql = format!("SELECT {} FROM {}", columns.join(", "), table_name);
    let source_rows = source_client.query(&select_sql, &[]).await?;
    
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        columns.join(", "),
        (1..=columns.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ")
    );
    
    for row in source_rows {
        let mut values: Vec<Box<dyn ToSql + Sync>> = Vec::new();
        for column in columns {
            let value: String = row.get(column.as_str());
            values.push(Box::new(value));
        }
        let params: Vec<&(dyn ToSql + Sync)> = values.iter().map(|v| v.as_ref()).collect();
        target_client.execute(&insert_sql, &params[..]).await?;
    }
    
    Ok(())
} 