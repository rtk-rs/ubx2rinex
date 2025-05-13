use rinex::prelude::{Duration, Epoch};

#[derive(Debug, Copy, Clone)]
pub struct Runtime {
    pub deploy_time: Epoch,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            deploy_time: {
                let now = Epoch::now().unwrap_or_else(|e| {
                    panic!("Failed to determine system time: {}", e);
                });

                now.round(Duration::from_seconds(1.0))
            },
        }
    }
}
