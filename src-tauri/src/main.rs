// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 使用库中导出的所有内容
use db_copier_lib::*;
// 使用更有组织的导入方式
use db_copier_lib::services::commands;
use db_copier_lib::services::TaskStore;
use db_copier_lib::services::Storage;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::info;
use tauri::Manager;
use tokio;

fn main() {
    // 使用库中的初始化函数
    init();
    info!("应用程序启动");
    
    let task_store = TaskStore(Arc::new(Mutex::new(HashMap::new())));

    // 创建一个运行时
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    // 初始化存储
    let storage = rt.block_on(async {
        let storage = Storage::new().await.expect("Failed to initialize storage");
        storage.init_db().await.expect("Failed to initialize database tables");
        storage
    });
    let storage = Arc::new(storage);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            app.manage(Arc::new(Mutex::new(HashMap::<String, Instant>::new())));
            app.manage(storage);
            
            let window = app.get_webview_window("main").unwrap();
            let start = Instant::now();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    let duration = start.elapsed();
                    info!("Window session duration: {:?}", duration);
                }
            });
            
            Ok(())
        })
        .manage(task_store)
        .invoke_handler(tauri::generate_handler![
            // 配置相关命令
            commands::list_configs,
            commands::load_config,
            commands::save_config,
            commands::delete_config,
            commands::import_config,
            // commands::export_config, // 暂时注释掉未实现的命令
            
            // 连接相关命令
            commands::test_connection,
            commands::get_tables,
            commands::get_table_columns,
            commands::get_table_info,
            commands::sync_table_structure,
            commands::compare_table_structure,
            
            // 任务相关命令
            commands::start_copy,
            commands::get_task_status,
            // commands::stop_task, // 暂时注释掉未实现的命令
            
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
