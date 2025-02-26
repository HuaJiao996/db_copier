// 数据库复制工具的核心库
// 导出所有公共模块和功能

// 导出主要模块
pub mod database;
pub mod utils;
pub mod services;

// 提供更有组织的访问方式
pub mod db {
    pub use crate::database::connection::*;
    pub use crate::database::model::*;
    pub use crate::database::schema::*;
}

pub mod service {
    pub use crate::services::command::*;
    pub use crate::services::store::*;
}

pub mod util {
    pub use crate::utils::logging::*;
}

// 初始化函数，可以在main.rs中调用
pub fn init() {
    utils::init_logger().expect("Failed to initialize logger");
}
