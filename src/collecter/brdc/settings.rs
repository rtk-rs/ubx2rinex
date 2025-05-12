use std::{collections::HashMap, str::FromStr};

use hifitime::{
    efmt::Format,
    prelude::{Duration, Epoch, Formatter, TimeScale},
};

use rinex::{
    prelude::{Constellation, Observable},
    production::{FFU, PPU},
};

#[derive(Debug, Clone)]
pub struct Settings {
    pub major: u8,
    pub gzip: bool,
    pub name: String,
    pub country: String,
    pub period: Duration,
    pub short_filename: bool,
    pub prefix: Option<String>,
    pub agency: Option<String>,
    pub operator: Option<String>,
}

impl Settings {
    pub fn filename(&self, t: Epoch) -> String {
        let mut filepath = if let Some(prefix) = &self.prefix {
            format!("{}/", prefix)
        } else {
            "".to_string()
        };

        let filename = if self.short_filename {
            self.v2_filename(t)
        } else {
            self.v3_filename(t)
        };

        filepath.push_str(&filename);
        filepath
    }

    fn v2_filename(&self, t: Epoch) -> String {
        let (y, _, _, _, _, _, _) = t.to_gregorian_utc();

        let fmt = Format::from_str("%j").unwrap();
        let formatter = Formatter::new(t, fmt);

        let mut formatted = self.name.to_string();

        formatted.push_str(&formatter.to_string());
        formatted.push('.');

        formatted.push_str(&format!("{:02}", y - 2000));
        formatted.push('N');

        if self.gzip {
            formatted.push_str(".gz")
        }

        formatted
    }

    fn v3_filename(&self, t: Epoch) -> String {
        let ppu: PPU = self.period.into();
        let ffu: FFU = Duration::from_seconds(30.0).into();

        let mut formatted = format!("{}{}_R_", self.name, self.country);

        let fmt = Format::from_str("%Y%j").unwrap();
        let formatter = Formatter::new(t, fmt);

        formatted.push_str(&formatter.to_string());
        formatted.push_str("0000_");

        formatted.push_str(&ppu.to_string());
        formatted.push('_');

        formatted.push_str(&ffu.to_string());
        formatted.push_str("_MN.rnx");

        if self.gzip {
            formatted.push_str(".gz");
        }

        formatted
    }
}

#[cfg(test)]
mod test {
    use super::Settings;
    use hifitime::prelude::{Duration, Epoch, TimeScale};
    use std::str::FromStr;

    #[test]
    fn test_v2_filename() {
        let mut settings = Settings {
            major: 3,
            agency: None,
            operator: None,
            gzip: false,
            prefix: None,
            short_filename: true,
            name: "UBX".to_string(),
            country: "FRA".to_string(),
            period: Duration::from_days(1.0),
        };

        let t0 = Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap();

        assert_eq!(settings.v2_filename(t0), "UBX001.20O");

        settings.gzip = true;
        assert_eq!(settings.v2_filename(t0), "UBX001.20D.gz");
    }

    #[test]
    fn test_v3_filename() {
        let mut settings = Settings {
            major: 3,
            agency: None,
            operator: None,
            gzip: false,
            prefix: None,
            short_filename: false,
            name: "UBX".to_string(),
            country: "FRA".to_string(),
            period: Duration::from_days(1.0),
        };

        let t0 = Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap();

        assert_eq!(
            settings.v3_filename(t0),
            "UBXFRA_R_20200010000_01D_30S_MO.rnx"
        );

        assert_eq!(
            settings.v3_filename(t0),
            "UBXFRA_R_20200010000_01D_30S_MO.crx"
        );

        settings.gzip = true;

        assert_eq!(
            settings.v3_filename(t0),
            "UBXFRA_R_20200010000_01D_30S_MO.crx.gz"
        );
    }
}
