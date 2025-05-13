use std::str::FromStr;

use hifitime::{
    efmt::Format,
    prelude::{Duration, Epoch, Formatter},
};

use rinex::production::{FFU, PPU};

use crate::collecter::Settings as SharedSettings;

#[derive(Debug, Clone)]
pub struct Settings {}

impl Settings {
    pub fn filename(&self, t: Epoch, shared_opts: &SharedSettings) -> String {
        let mut filepath = if let Some(prefix) = &shared_opts.prefix {
            format!("{}/", prefix)
        } else {
            "".to_string()
        };

        let filename = if shared_opts.long_filename {
            self.v3_filename(t, shared_opts)
        } else {
            self.v2_filename(t, shared_opts)
        };

        filepath.push_str(&filename);
        filepath
    }

    fn v2_filename(&self, t: Epoch, shared_opts: &SharedSettings) -> String {
        let (y, _, _, _, _, _, _) = t.to_gregorian_utc();

        let fmt = Format::from_str("%j").unwrap();
        let formatter = Formatter::new(t, fmt);

        let mut formatted = shared_opts.name.to_string();

        formatted.push_str(&formatter.to_string());
        formatted.push('.');

        formatted.push_str(&format!("{:02}", y - 2000));
        formatted.push('N');

        if shared_opts.gzip {
            formatted.push_str(".gz")
        }

        formatted
    }

    fn v3_filename(&self, t: Epoch, shared_opts: &SharedSettings) -> String {
        let ppu: PPU = shared_opts.snapshot_period.into();
        let ffu: FFU = Duration::from_seconds(30.0).into();

        let mut formatted = format!("{}{}_R_", shared_opts.name, shared_opts.country_code);

        let fmt = Format::from_str("%Y%j").unwrap();
        let formatter = Formatter::new(t, fmt);

        formatted.push_str(&formatter.to_string());
        formatted.push_str("0000_");

        formatted.push_str(&ppu.to_string());
        formatted.push('_');

        formatted.push_str(&ffu.to_string());
        formatted.push_str("_MN.rnx");

        if shared_opts.gzip {
            formatted.push_str(".gz");
        }

        formatted
    }
}

#[cfg(test)]
mod test {

    use crate::collecter::{
        brdc::settings::Settings as NavSettings, settings::Settings as SharedSettings,
    };

    use hifitime::prelude::{Duration, Epoch};
    use std::str::FromStr;

    #[test]
    fn test_v2_filename() {
        let mut shared = SharedSettings {
            major: 3,
            agency: None,
            operator: None,
            gzip: false,
            no_obs: false,
            nav: true,
            prefix: None,
            long_filename: false,
            name: "UBX".to_string(),
            country_code: "FRA".to_string(),
            snapshot_period: Duration::from_days(1.0),
        };

        let settings = NavSettings {};

        let t0 = Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap();

        assert_eq!(settings.v2_filename(t0, &shared), "UBX001.20O");

        shared.gzip = true;
        assert_eq!(settings.v2_filename(t0, &shared), "UBX001.20D.gz");
    }

    #[test]
    fn test_v3_filename() {
        let mut shared = SharedSettings {
            major: 3,
            agency: None,
            operator: None,
            gzip: false,
            no_obs: false,
            nav: true,
            prefix: None,
            long_filename: true,
            name: "UBX".to_string(),
            country_code: "FRA".to_string(),
            snapshot_period: Duration::from_days(1.0),
        };

        let settings = NavSettings {};

        let t0 = Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap();

        assert_eq!(
            settings.v3_filename(t0, &shared),
            "UBXFRA_R_20200010000_01D_30S_MO.rnx"
        );

        assert_eq!(
            settings.v3_filename(t0, &shared),
            "UBXFRA_R_20200010000_01D_30S_MO.crx"
        );

        shared.gzip = true;

        assert_eq!(
            settings.v3_filename(t0, &shared),
            "UBXFRA_R_20200010000_01D_30S_MO.crx.gz"
        );
    }
}
