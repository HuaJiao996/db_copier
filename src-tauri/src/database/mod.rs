// 数据库相关功能的模块

// 子模块
mod db;
mod models;

pub use db::*;
pub use models::*;

// 数据库连接器
pub mod connection {
    use super::*;
    
    pub use db::DbCopier;
    pub use db::DbError;
}

// 数据库模型
pub mod model {
    use super::*;
    
    pub use models::Config;
    pub use models::DatabaseConfig;
    pub use models::SSHConfig;
    pub use models::TableConfig;
    pub use models::MaskRule;
    pub use models::MaskRuleType;
    pub use models::TaskStatus;
    pub use models::TaskState;
    pub use models::Progress;
}

// 表结构相关
pub mod schema {
    use super::*;
    
    pub use db::TableInfo;
    pub use db::ColumnInfo;
} 