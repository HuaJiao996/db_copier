use crate::database::{Config, DbError, MaskRule, MaskRuleType, TableConfig};
use crate::db::DbClient;
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;

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
    source_client: DbClient,
    target_client: DbClient,
    table_info_cache: Arc<RwLock<HashMap<String, TableInfo>>>,
}

impl DbCopier {
    pub async fn new(config: &Config) -> Result<Self, DbError> {
        // 检查源数据库配置是否有效
        let source_valid = !config.source_db.host.is_empty()
            && !config.source_db.database.is_empty()
            && !config.source_db.username.is_empty();

        // 检查目标数据库配置是否有效
        let target_valid = !config.target_db.host.is_empty()
            && !config.target_db.database.is_empty()
            && !config.target_db.username.is_empty();

        if !source_valid || !target_valid {
            return Err(DbError::Connection("数据库配置无效".to_string()));
        }

        let source_client = DbClient::new(&config.source_db).await?;
        let target_client = DbClient::new(&config.target_db).await?;

        Ok(Self {
            source_client,
            target_client,
            table_info_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    #[allow(dead_code)]
    fn apply_mask_rule(&self, value: &str, rule: &MaskRule) -> String {
        match rule.rule_type {
            MaskRuleType::Hash => {
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            MaskRuleType::Fixed => rule.pattern.clone().unwrap_or_else(|| "****".to_string()),
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
                            }
                            '*' => {
                                if chars.next().is_some() {
                                    result.push('*');
                                }
                            }
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
    async fn execute_batch_insert(
        &self,
        table: &TableConfig,
        batch_values: &mut Vec<String>,
        batch_params: &mut Vec<String>,
    ) -> Result<(), DbError> {
        if batch_values.is_empty() {
            return Ok(());
        }

        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table.name,
            table
                .columns
                .iter()
                .map(|c| c.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
            batch_values.join(", ")
        );

        self.target_client
            .client
            .execute(
                &insert_sql,
                &batch_params.iter().map(|s| s as _).collect::<Vec<_>>(),
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        batch_values.clear();
        batch_params.clear();

        Ok(())
    }
    #[allow(dead_code)]
    async fn copy_table_data_stream(&self, table: &TableConfig) -> Result<(), DbError> {
        const BATCH_SIZE: usize = 1000;
        let mut values: Vec<String> = Vec::new();
        let mut batch_params: Vec<String> = Vec::new();
        let mut value_count = 0;

        // 构建查询语句
        let columns = table
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>()
            .join(", ");
        let select_sql = format!("SELECT {} FROM {}", columns, table.name);

        // 执行查询
        let source_rows = self
            .source_client
            .client
            .query(&select_sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // 构建插入语句
        let insert_sql = format!("INSERT INTO {} ({}) VALUES", table.name, columns);

        let client = &self.target_client.client;

        // 处理每一行数据
        for row in source_rows {
            let mut row_placeholders = Vec::new();
            for column in table.columns.iter() {
                value_count += 1;
                let value: String = row.get(column.name.as_str());
                values.push(value);
                row_placeholders.push(format!("${}", value_count));
            }
            batch_params.push(format!("({})", row_placeholders.join(", ")));

            if batch_params.len() >= BATCH_SIZE {
                let full_insert_sql = format!("{} {}", insert_sql, batch_params.join(", "));
                let params: Vec<&(dyn ToSql + Sync)> =
                    values.iter().map(|v| v as &(dyn ToSql + Sync)).collect();
                client
                    .execute(&full_insert_sql, &params[..])
                    .await
                    .map_err(|e| DbError::Query(e.to_string()))?;

                values.clear();
                batch_params.clear();
                value_count = 0;
            }
        }

        // 处理剩余的数据
        if !batch_params.is_empty() {
            let full_insert_sql = format!("{} {}", insert_sql, batch_params.join(", "));
            let params: Vec<&(dyn ToSql + Sync)> =
                values.iter().map(|v| v as &(dyn ToSql + Sync)).collect();
            client
                .execute(&full_insert_sql, &params[..])
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn copy_table(&self, table: &TableConfig) -> Result<(), DbError> {
        // 获取并同步表结构
        let table_info = self.get_table_info(&table.name).await?;
        self.sync_table_structure(&table_info, table.ignore_foreign_keys)
            .await?;

        // 如果只复制结构，则直接返回
        if table.structure_only {
            return Ok(());
        }

        // 使用流式处理复制数据
        let columns = table
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>()
            .join("\", \"");
        let select_sql = format!("SELECT \"{}\" FROM \"{}\"", columns, table.name);

        // 执行查询并插入数据
        let rows = self
            .source_client
            .client
            .query(&select_sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        if !rows.is_empty() {
            let insert_sql = format!("INSERT INTO \"{}\" (\"{}\") VALUES", table.name, columns);

            // 构建批量插入的值
            let mut values = Vec::new();
            let mut params = Vec::new();
            let mut param_count = 1;

            for row in rows {
                let mut row_values = Vec::new();
                for column in &table.columns {
                    let value: String = row.get(column.name.as_str());
                    params.push(value);
                    row_values.push(format!("${}", param_count));
                    param_count += 1;
                }
                values.push(format!("({})", row_values.join(", ")));
            }

            // 执行批量插入
            let full_insert_sql = format!("{} {}", insert_sql, values.join(", "));
            self.target_client
                .client
                .execute(
                    &full_insert_sql,
                    &params
                        .iter()
                        .map(|s| s as &(dyn ToSql + Sync))
                        .collect::<Vec<_>>(),
                )
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

        let row = self
            .source_client
            .client
            .query_one(&sql, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let create_sql: String = row.get("create_sql");
        Ok(create_sql)
    }

    

    

    pub async fn get_table_info(&self, table_name: &str) -> Result<TableInfo, DbError> {
        // 先检查缓存
        if let Some(info) = self.table_info_cache.read().await.get(table_name) {
            return Ok(info.clone());
        }

        // 如果缓存中没有，则从数据库获取
        let rows = self.source_client.client
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
        self.table_info_cache
            .write()
            .await
            .insert(table_name.to_string(), info.clone());

        Ok(info)
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<String>, DbError> {
        let rows = self
            .source_client
            .client
            .query(
                "SELECT indexdef FROM pg_indexes WHERE tablename = $1",
                &[&table_name],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<String>, DbError> {
        let rows = self.source_client.client
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

    pub async fn sync_table_structure(
        &self,
        table_info: &TableInfo,
        ignore_foreign_keys: bool,
    ) -> Result<(), DbError> {
        // 删除目标表(如果存在)
        self.target_client
            .client
            .execute(
                &format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table_info.name),
                &[],
            )
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // 创建表
        let mut create_table = format!("CREATE TABLE \"{}\" (\n", table_info.name);

        // 添加列定义
        let column_defs: Vec<String> = table_info
            .columns
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
            .client
            .execute(&create_table, &[])
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // 添加约束（如果不忽略外键）
        if !ignore_foreign_keys {
            for constraint in &table_info.constraints {
                // 跳过外键约束
                if !constraint.contains("FOREIGN KEY") {
                    self.target_client
                        .client
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
                .client
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

    #[allow(dead_code)]
    pub async fn copy_table_data(
        source_client: &Client,
        target_client: &Client,
        table_name: &str,
        columns: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let select_sql = format!("SELECT {} FROM {}", columns.join(", "), table_name);
        let source_rows = source_client.query(&select_sql, &[]).await?;

        let insert_sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_name,
            columns.join(", "),
            (1..=columns.len())
                .map(|i| format!("${}", i))
                .collect::<Vec<_>>()
                .join(", ")
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
}
