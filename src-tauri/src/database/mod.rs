// 数据库相关功能的模块

// 子模块
mod models;
mod db_client;

pub use db_client::*;
pub use models::*;

// 数据库连接器
pub mod connection {
    use super::*;
    
    pub use db_client::DbClient;
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
