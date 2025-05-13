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

    /// Custom name (usually receiver model, or marker name)
    pub name: String,

    /// RINEX revision major number
    pub major: u8,

    /// File prefix
    pub prefix: Option<String>,

    /// Agency name
    pub agency: Option<String>,

    /// Agency country code
    pub country_code: String,

    /// Operator / observer
    pub operator: Option<String>,

    /// Snapshot period
    pub snapshot_period: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            prefix: None,
            agency: None,
            operator: None,
            name: "UBLX".to_string(),
            country_code: "XXX".to_string(),
            no_obs: false,
            nav: false,
            gzip: false,
            long_filename: false,
            major: 3,
            snapshot_period: Duration::from_hours(1.0),
        }
    }
}
