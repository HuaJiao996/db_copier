use crate::database::{DatabaseConfig, SSHConfig};
use log::{error, info};
use postgres_native_tls;
use ssh2::Session;
use std::error::Error;
use std::fmt;
use std::net::TcpStream;
use tokio::time::Duration;
use tokio_postgres::{Client, Config as PgConfig};

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

pub struct DbClient {
    pub client: Client,
    pub ssh_session: Option<Session>,
}

impl DbClient {
    pub async fn new(database_config: &DatabaseConfig) -> Result<Self, DbError> {
        let (client, ssh_session) = match &database_config.ssh_config {
            Some(ssh_config) => {
                let (client, session) =
                    Self::connect_with_ssh(ssh_config, &database_config).await?;
                (client, Some(session))
            }
            None => {
                let client = Self::connect_db(&database_config).await?;
                (client, None)
            }
        };

        Ok(Self {
            client,
            ssh_session,
        })
    }

    async fn connect_with_ssh(
        ssh_config: &SSHConfig,
        db_config: &DatabaseConfig,
    ) -> Result<(Client, Session), DbError> {
        info!("开始建立 SSH 连接 {}:{}", ssh_config.host, ssh_config.port);

        let tcp = TcpStream::connect(format!("{}:{}", ssh_config.host, ssh_config.port)).map_err(
            |e| {
                error!("SSH TCP 连接失败: {}", e);
                DbError::SSH(e.to_string())
            },
        )?;

        info!("创建 SSH 会话");
        let mut session = Session::new().map_err(|e| {
            error!("创建 SSH 会话失败: {}", e);
            DbError::SSH(e.to_string())
        })?;
        session.set_tcp_stream(tcp);

        info!("进行 SSH 握手");
        session.handshake().map_err(|e| {
            error!("SSH 握手失败: {}", e);
            DbError::SSH(e.to_string())
        })?;

        // SSH认证
        info!("开始 SSH 认证");
        match ssh_config.auth_type.as_str() {
            "password" => {
                info!("使用密码认证");
                if let Some(password) = &ssh_config.password {
                    info!("使用密码认证");
                    session
                        .userauth_password(&ssh_config.username, password)
                        .map_err(|e| {
                            error!("SSH 密码认证失败: {}", e);
                            DbError::SSH(e.to_string())
                        })?;
                } else {
                    return Err(DbError::SSH("密码不能为空".to_string()));
                }
            }
            "private_key" => {
                if let Some(key_path) = &ssh_config.private_key_path {
                    info!("使用密钥认证: {}", key_path);
                    session
                        .userauth_pubkey_file(
                            &ssh_config.username,
                            None,
                            std::path::Path::new(key_path),
                            None,
                        )
                        .map_err(|e| {
                            error!("SSH 密钥认证失败: {}", e);
                            DbError::SSH(e.to_string())
                        })?;
                } else {
                    return Err(DbError::SSH("密钥路径不能为空".to_string()));
                }
            }
            _ => {
                return Err(DbError::SSH("不支持的认证方式".to_string()));
            }
        }

        info!("设置 SSH 端口转发");
        let local_port = Self::find_available_port()?;
        info!("使用本地端口: {}", local_port);
        let _listener = session
            .channel_forward_listen(local_port, None, None)
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
        let port = listener
            .local_addr()
            .map_err(|e| DbError::Connection(e.to_string()))?
            .port();
        Ok(port)
    }

    async fn connect_db(config: &DatabaseConfig) -> Result<Client, DbError> {
        info!(
            "开始连接数据库 {}:{}/{}",
            config.host, config.port, config.database
        );
        info!("SSL 模式: {}", config.ssl_mode);

        // 验证连接参数
        if config.host.is_empty() {
            return Err(DbError::Connection("数据库主机地址不能为空".to_string()));
        }

        if config.database.is_empty() {
            return Err(DbError::Connection("数据库名不能为空".to_string()));
        }

        if config.username.is_empty() {
            return Err(DbError::Connection("数据库用户名不能为空".to_string()));
        }

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
                }
                "disable" => {
                    info!("禁用 SSL 连接");
                    tokio_postgres::config::SslMode::Disable
                }
                _ => {
                    info!("使用首选 SSL 连接");
                    tokio_postgres::config::SslMode::Prefer
                }
            });

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

        info!(
            "建立数据库连接 {}:{}/{}",
            config.host, config.port, config.database
        );
        let (client, connection) = pg_config.connect(connector).await.map_err(|e| {
            error!("连接数据库失败 ({}:{}): {}", config.host, config.port, e);
            DbError::Connection(format!(
                "连接数据库失败 ({}:{}): {}",
                config.host, config.port, e
            ))
        })?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("数据库连接错误: {}", e);
            }
        });

        info!(
            "数据库连接成功 {}:{}/{}",
            config.host, config.port, config.database
        );
        Ok(client)
    }

    pub async fn get_tables(&self) -> Result<Vec<String>, DbError> {
        let rows = self
            .client
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
        let query = "
            SELECT column_name 
            FROM information_schema.columns 
            WHERE table_name = $1 
            ORDER BY ordinal_position
        ";

        let rows = self
            .client
            .query(query, &[&table_name])
            .await
            .map_err(|e| DbError::Query(format!("获取表列失败: {}", e)))?;

        let columns = rows.iter().map(|row| row.get::<_, String>(0)).collect();

        Ok(columns)
    }
}
