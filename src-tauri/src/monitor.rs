use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use log::info;

pub struct MemoryMonitor {
    usage: Arc<AtomicUsize>,
    interval_secs: u64,
    threshold_mb: usize,
}

impl MemoryMonitor {
    pub fn new(interval_secs: u64, threshold_mb: usize) -> Self {
        Self {
            usage: Arc::new(AtomicUsize::new(0)),
            interval_secs,
            threshold_mb,
        }
    }

    #[allow(dead_code)]
    pub fn get_usage(&self) -> usize {
        self.usage.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn update_usage(&self, delta: isize) {
        if delta >= 0 {
            self.usage.fetch_add(delta as usize, Ordering::Relaxed);
        } else {
            self.usage.fetch_sub((-delta) as usize, Ordering::Relaxed);
        }
    }

    pub async fn start_monitoring(self: Arc<Self>) {
        let usage = self.usage.clone();
        let threshold = self.threshold_mb * 1024 * 1024;
        let interval = Duration::from_secs(self.interval_secs);

        tokio::spawn(async move {
            loop {
                let current_usage = usage.load(Ordering::Relaxed);
                if current_usage > threshold {
                    info!("Warning: Memory usage ({} MB) exceeds threshold ({} MB)",
                        current_usage / 1024 / 1024,
                        threshold / 1024 / 1024
                    );
                }
                tokio::time::sleep(interval).await;
            }
        });
    }
} 