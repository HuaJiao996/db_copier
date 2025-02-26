use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use chrono::Local;

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();
    
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}:{} - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    Ok(())
} 