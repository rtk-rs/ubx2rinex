use rinex::prelude::Epoch;

#[derive(Debug, Copy, Clone)]
pub struct Runtime {
    pub deploy_time: Epoch,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            deploy_time: {
                Epoch::now().unwrap_or_else(|e| {
                    panic!("Failed to determine system time: {}", e);
                })
            },
        }
    }
}
