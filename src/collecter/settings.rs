use rinex::prelude::Duration;

/// [Settings] shared by all data collectors
#[derive(Debug, Clone)]
pub struct Settings {
    /// OBS collection should be disabled
    pub no_obs: bool,

    /// NAV collection requested
    pub nav: bool,

    /// GZIp compression
    pub gzip: bool,

    /// Prefer long file names
    pub long_filename: bool,

    pub name: String,
    pub major: u8,
    pub country_code: String,
    pub prefix: Option<String>,
    pub agency: Option<String>,
    pub operator: Option<String>,
    pub snapshot_period: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            prefix: None,
            agency: None,
            operator: None,
            name: "UBX".to_string(),
            country_code: "FRA".to_string(),
            no_obs: false,
            nav: false,
            gzip: false,
            long_filename: false,
            major: 3,
            snapshot_period: Duration::from_hours(1.0),
        }
    }
}