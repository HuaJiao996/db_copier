// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod db;
mod commands;
mod logger;
mod monitor;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use commands::{
    test_connection, start_copy, get_task_status, get_table_columns,
    get_tables, save_config, load_config, list_configs, delete_config,
    get_table_info, sync_table_structure, get_memory_usage, TaskStore
};
use std::time::Instant;
use monitor::MemoryMonitor;
use log::info;
use tauri::Manager;
use tokio;

fn main() {
    logger::init_logger().expect("Failed to initialize logger");
    info!("应用程序启动");
    
    let task_store = TaskStore(Arc::new(Mutex::new(HashMap::new())));

    // 初始化内存监控
    let memory_monitor = Arc::new(MemoryMonitor::new(5, 100)); // 每5秒检查一次，阈值100MB
    let monitor_clone = memory_monitor.clone();

    // 创建一个运行时
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();

    // 在运行时中启动监控
    rt.spawn(async move {
        monitor_clone.start_monitoring().await;
    });

    tauri::Builder::default()
        .setup(move |app| {
            app.manage(Arc::new(Mutex::new(HashMap::<String, Instant>::new())));
            app.manage(memory_monitor.clone());
            
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
            test_connection,
            start_copy,
            get_task_status,
            get_table_columns,
            get_tables,
            save_config,
            load_config,
            list_configs,
            delete_config,
            get_table_info,
            sync_table_structure,
            get_memory_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
