use std::{collections::HashMap, str::FromStr};

use hifitime::{
    efmt::Format,
    prelude::{Duration, Epoch, Formatter, TimeScale},
};

use rinex::{
    prelude::{Constellation, Observable},
    production::{FFU, PPU},
};

use crate::collecter::settings::Settings as SharedSettings;

/// Observation RINEX collection [Settings]
#[derive(Debug, Clone)]
pub struct Settings {
    pub crinex: bool,
    pub dop: bool,
    pub cp: bool,
    pub pr: bool,
    pub timescale: TimeScale,
    pub observables: HashMap<Constellation, Vec<Observable>>,
}
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
        let year = t.to_gregorian_utc().0;

        let fmt = Format::from_str("%j").unwrap();
        let formatter = Formatter::new(t, fmt);

        let mut formatted = shared_opts.name.to_string();

        formatted.push_str(&formatter.to_string());
        formatted.push('.');
        formatted.push_str(&format!("{:02}", year - 2000));

        if self.crinex {
            formatted.push('D');
        } else {
            formatted.push('O');
        }

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
        formatted.push_str("_MO");

        if self.crinex {
            formatted.push_str(".crx");
        } else {
            formatted.push_str(".rnx");
        }

        if shared_opts.gzip {
            formatted.push_str(".gz");
        }

        formatted
    }
}

#[cfg(test)]
mod test {

    use crate::collecter::{
        observations::settings::Settings, settings::Settings as SharedSettings,
    };

    use hifitime::prelude::{Duration, Epoch, TimeScale};

    use std::str::FromStr;

    #[test]
    fn test_v2_filename() {
        let mut shared = SharedSettings::default();
        shared.long_filename = false;
        shared.snapshot_period = Duration::from_days(1.0);

        let mut settings = Settings {
            dop: false,
            pr: false,
            cp: false,
            crinex: false,
            timescale: TimeScale::GPST,
            observables: Default::default(),
        };

        let t0 = Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap();

        assert_eq!(settings.v2_filename(t0, &shared), "UBX001.20O");

        settings.crinex = true;
        assert_eq!(settings.v2_filename(t0, &shared), "UBX001.20D");

        shared.gzip = true;
        assert_eq!(settings.v2_filename(t0, &shared), "UBX001.20D.gz");
    }

    #[test]
    fn test_v3_filename() {
        let mut shared = SharedSettings::default();

        shared.major = 3;
        shared.long_filename = true;
        shared.snapshot_period = Duration::from_days(1.0);

        let mut settings = Settings {
            dop: false,
            pr: false,
            cp: false,
            crinex: false,
            timescale: TimeScale::GPST,
            observables: Default::default(),
        };

        let t0 = Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap();

        assert_eq!(
            settings.v3_filename(t0, &shared),
            "UBXFRA_R_20200010000_01D_30S_MO.rnx"
        );

        settings.crinex = true;

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
