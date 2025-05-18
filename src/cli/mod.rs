use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};
use rinex::prelude::{Constellation, Duration, Observable, TimeScale};

mod observations;
use observations::*;

mod navigation;
use navigation::*;

use crate::{
    collecter::{
        brdc::settings::Settings as NavSettings, observations::settings::Settings as ObsSettings,
        settings::Settings as SharedSettings,
    },
    UbloxSettings,
};

use std::{collections::HashMap, str::FromStr};

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        let cmd =
            Command::new("ubx2rinex")
        .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
        .version(env!("CARGO_PKG_VERSION"))
        .about("U-Blox to RINEX Files collecter")
        .color(ColorChoice::Always)
        .arg_required_else_help(true)
        .next_help_heading("Serial port")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .required(true)
                .help("Define serial port. Example /dev/ttyUSB0 on Linux")
        )
        .arg(
            Arg::new("baudrate")
                .short('b')
                .long("baud")
                .required(false)
                .value_name("Baudrate (u32)")
                .help("Define serial port baud rate. Communications will not work if your U-Blox streams at a different data-rate. By default we use 115_200"),
        )
        .next_help_heading("Constellation configuration")
        .arg(
            Arg::new("gps")
                .long("gps")
                .action(ArgAction::SetTrue)
                .help("Activate GPS constellation")
        )
        .arg(
            Arg::new("galileo")
                .long("galileo")
                .action(ArgAction::SetTrue)
                .help("Activate Galileo constellation")
        )
        .arg(
            Arg::new("bds")
                .long("bds")
                .action(ArgAction::SetTrue)
                .help("Activate BDS (BeiDou) constellation")
        )
        .arg(
            Arg::new("qzss")
                .long("qzss")
                .action(ArgAction::SetTrue)
                .help("Activate QZSS constellation")
        )
        .arg(
            Arg::new("glonass")
                .long("glonass")
                .action(ArgAction::SetTrue)
                .help("Activate Glonass constellation")
        )
        .next_help_heading("Signal selection - at least one required!")
        .arg(
            Arg::new("l1")
                .long("l1")
                .action(ArgAction::SetTrue)
                .help("Activate L1 signal for all constellations")
                .required_unless_present_any(["l2", "l5"]),
        )
        .arg(
            Arg::new("l2")
                .long("l2")
                .action(ArgAction::SetTrue)
                .help("Activate L2 signal for all constellations")
                .required_unless_present_any(["l1", "l5"]),
        )
        .arg(
            Arg::new("l5")
                .long("l5")
                .action(ArgAction::SetTrue)
                .help("Activate L5 signal for all constellations. Requires F9 or F10 series.")
                .required_unless_present_any(["l1", "l2"]),
        )
        .next_help_heading("U-Blox configuration")
        .arg(
            Arg::new("profile")
                .long("prof")
                .action(ArgAction::Set)
                .help("Define user profile. Default is set to \"portable\""),
        )
        .arg(
            Arg::new("rx-clock")
                .long("rx-clock")
                .action(ArgAction::SetTrue)
                .help("Resolve & collect clock state. Disabled by default"),
        )
        .arg(
            Arg::new("anti-spoofing")
                .long("anti-spoofing")
                .action(ArgAction::SetTrue)
                .help("Makes sure anti jamming/spoofing is enabled. When enabled, it is automatically emphasized in the collected RINEX."))
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .required(false)
                .value_name("Model")
                .help("Define u-Blox receiver model. For example \"u-Blox M8T\"")
        )
        .next_help_heading("RINEX Collection (shared)")
        .arg(
            Arg::new("name")
                .long("name")
                .short('n')
                .required(false)
                .action(ArgAction::Set)
                .help("Define a custom name. To respect standard naming conventions,
this should be a 4 letter code, usually named after your geodetic marker or receiver model.
When not defined, the default value is \"UBXR\".")
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .required(false)
                .help("Custom directory prefix for output products. Default is none."),
        )
        .arg(
            Arg::new("country")
                .short('c')
                .action(ArgAction::Set)
                .help("Specify country code (3 letter) in case of V3 file name. Default: \"FRA\"")
        )
        .arg(
            Arg::new("agency")
                .long("agency")
                .action(ArgAction::Set)
                .required(false)
                .help("Define name of your Agency, to be used in all Headers"),
        )
        .arg(
            Arg::new("observer")
                .long("observer")
                .action(ArgAction::Set)
                .required(false)
                .help("Define name of Observer, to be used in all Headers"),
        )
        .arg(
            Arg::new("operator")
                .long("operator")
                .action(ArgAction::Set)
                .required(false)
                .help("Define name of Operator, to be used in all Headers"),
        )
        .arg(
            Arg::new("v2")
                .long("v2")
                .action(ArgAction::SetTrue)
                .help("Collect RINEX V2. Default is RINEX V3."),
        )
        .arg(
            Arg::new("v4")
                .long("v4")
                .action(ArgAction::SetTrue)
                .help("Collect RINEX V4. Default is RINEX V3."),
        )
        .arg(
            Arg::new("long")
                .short('l')
                .long("long")
                .action(ArgAction::SetTrue)
                .help("Prefer V3 (modern) longer filenames, instead of shortened (default).
Requires country code and other definitions to be complete.")
        )
        .arg(
            Arg::new("gzip")
                .long("gzip")
                .action(ArgAction::SetTrue)
                .help("Add GZIP compression.")
        )
        .arg(
            Arg::new("hourly")
                .long("hourly")
                .action(ArgAction::SetTrue)
                .required(false)
                .conflicts_with("half-day")
                .help("Hourly snapshot (one file release every hour). Default is 24 hours (daily).")
        )
        .arg(
            Arg::new("half-day")
                .long("half-day")
                .action(ArgAction::SetTrue)
                .required(false)
                .conflicts_with("hourly")
                .help("2x Daily snapshot (one file release every 12 hours). Default is 24 hours (daily).")
        )
        .arg(
            Arg::new("nav")
                .long("nav")
                .action(ArgAction::SetTrue)
                .help("Activate NAV RINEX collection (disabled by default)."),
        )
        .arg(
            Arg::new("no-obs")
                .long("no-obs")
                .action(ArgAction::SetTrue)
                .help("Disable Observation RINEX collection. 
Use this if you intend to collect Ephemeris only."),
        );

        let cmd = cmd
            .next_help_heading("Observables (physics - required for Observation collection)")
            .args(OBSERVABLES_ARGS.iter());

        let cmd = cmd
            .next_help_heading("Observations RINEX (specific)")
            .args(OBSERVATION_ARGS.iter());

        let cmd = cmd
            .next_help_heading("NAV RINEX (specific)")
            .args(NAVIGATION_ARGS.iter());

        Self {
            matches: cmd.get_matches(),
        }
    }

    /// Returns User serial port specification
    pub fn port(&self) -> &str {
        self.matches.get_one::<String>("port").unwrap()
    }

    /// Returns User baud rate specification
    pub fn baud_rate(&self) -> Option<u32> {
        let baud = self.matches.get_one::<String>("baudrate")?;
        let baud = baud
            .parse::<u32>()
            .unwrap_or_else(|e| panic!("Invalid baud rate value: {}", e));
        Some(baud)
    }

    fn gps(&self) -> bool {
        self.matches.get_flag("gps")
    }

    fn galileo(&self) -> bool {
        self.matches.get_flag("galileo")
    }

    fn bds(&self) -> bool {
        self.matches.get_flag("bds")
    }

    fn qzss(&self) -> bool {
        self.matches.get_flag("qzss")
    }

    fn glonass(&self) -> bool {
        self.matches.get_flag("glonass")
    }

    fn constellations(&self) -> Vec<Constellation> {
        let mut constellations = Vec::<Constellation>::with_capacity(4);

        if self.gps() {
            constellations.push(Constellation::GPS);
        }
        if self.galileo() {
            constellations.push(Constellation::Galileo);
        }
        if self.bds() {
            constellations.push(Constellation::BeiDou);
        }
        if self.qzss() {
            constellations.push(Constellation::QZSS);
        }
        if self.glonass() {
            constellations.push(Constellation::Glonass);
        }
        constellations
    }

    fn l1(&self) -> bool {
        self.matches.get_flag("l1")
    }

    fn l2(&self) -> bool {
        self.matches.get_flag("l2")
    }

    fn l5(&self) -> bool {
        self.matches.get_flag("l5")
    }

    fn doppler(&self) -> bool {
        self.matches.get_flag("all-meas") || self.matches.get_flag("dop")
    }

    fn pr(&self) -> bool {
        self.matches.get_flag("all-meas") || self.matches.get_flag("pr")
    }

    fn cp(&self) -> bool {
        self.matches.get_flag("all-meas") || self.matches.get_flag("cp")
    }

    fn observables(&self) -> HashMap<Constellation, Vec<Observable>> {
        let v2 = self.matches.get_flag("v2");

        let mut ret = HashMap::<Constellation, Vec<Observable>>::new();

        for constell in self.constellations().iter() {
            let mut observables = Vec::new();

            if self.l1() {
                if self.doppler() {
                    if v2 {
                        observables.push("D1");
                    } else {
                        observables.push("D1C");
                    }
                }
                if self.cp() {
                    if v2 {
                        observables.push("L1");
                    } else {
                        observables.push("L1C");
                    }
                }
                if self.pr() {
                    if v2 {
                        observables.push("C1");
                    } else {
                        observables.push("C1C");
                    }
                }
            }

            if self.l2() {
                if self.doppler() {
                    if v2 {
                        observables.push("D2");
                    } else {
                        observables.push("D2C");
                    }
                }
                if self.cp() {
                    if v2 {
                        observables.push("L2");
                    } else {
                        observables.push("L2C");
                    }
                }
                if self.pr() {
                    if v2 {
                        observables.push("C2");
                    } else {
                        observables.push("C2C");
                    }
                }
            }

            if self.l5() {
                if self.doppler() {
                    if v2 {
                        observables.push("D5");
                    } else {
                        observables.push("D5C");
                    }
                }
                if self.cp() {
                    if v2 {
                        observables.push("L5");
                    } else {
                        observables.push("L5C");
                    }
                }
                if self.pr() {
                    if v2 {
                        observables.push("C5");
                    } else {
                        observables.push("C5C");
                    }
                }
            }

            for observable in observables.iter() {
                let observable = Observable::from_str(observable).unwrap();
                if let Some(observables) = ret.get_mut(constell) {
                    observables.push(observable);
                } else {
                    ret.insert(*constell, vec![observable]);
                }
            }
        }

        ret
    }

    fn timescale(&self) -> TimeScale {
        if let Some(ts) = self.matches.get_one::<String>("timescale") {
            let ts = TimeScale::from_str(ts.trim())
                .unwrap_or_else(|e| panic!("Invalid timescale: {}", e));
            ts
        } else {
            TimeScale::GPST
        }
    }

    fn sampling_period(&self) -> Duration {
        if let Some(sampling) = self.matches.get_one::<String>("sampling") {
            let dt = sampling
                .trim()
                .parse::<Duration>()
                .unwrap_or_else(|e| panic!("Invalid duration: {}", e));

            if dt.total_nanoseconds() < 50_000_000 {
                panic!("Sampling period is limited to 50ms");
            }
            dt
        } else {
            Duration::from_milliseconds(30_000.0)
        }
    }

    fn solutions_ratio(sampling_period: Duration) -> u16 {
        let period_ms = (sampling_period.total_nanoseconds() / 1_000_000) as u16;
        if period_ms > 10_000 {
            1
        } else if period_ms > 1_000 {
            2
        } else {
            10
        }
    }

    pub fn ublox_settings(&self) -> UbloxSettings {
        let sampling_period = self.sampling_period();
        UbloxSettings {
            l1: self.l1(),
            l2: self.l2(),
            l5: self.l5(),
            sampling_period,
            timescale: self.timescale(),
            constellations: self.constellations(),
            rx_clock: self.matches.get_flag("rx-clock"),
            solutions_ratio: Self::solutions_ratio(sampling_period),
            sn: None,
            firmware: None,
            model: if let Some(model) = self.matches.get_one::<String>("model") {
                Some(model.to_string())
            } else {
                None
            },
        }
    }

    pub fn settings(&self) -> SharedSettings {
        SharedSettings {
            agency: if let Some(agency) = self.matches.get_one::<String>("agency") {
                Some(agency.to_string())
            } else {
                None
            },
            operator: if let Some(operator) = self.matches.get_one::<String>("operator") {
                Some(operator.to_string())
            } else {
                None
            },
            prefix: if let Some(prefix) = self.matches.get_one::<String>("prefix") {
                Some(prefix.to_string())
            } else {
                None
            },
            name: if let Some(name) = self.matches.get_one::<String>("name") {
                name.to_string()
            } else {
                "UBX".to_string()
            },
            country_code: if let Some(country) = self.matches.get_one::<String>("country") {
                country.to_string()
            } else {
                "XXX".to_string()
            },
            gzip: self.matches.get_flag("gzip"),
            long_filename: self.matches.get_flag("long"),
            major: if self.matches.get_flag("v4") {
                4
            } else if self.matches.get_flag("v2") {
                2
            } else {
                3
            },
            snapshot_period: if self.matches.get_flag("hourly") {
                Duration::from_hours(1.0)
            } else if self.matches.get_flag("half-day") {
                Duration::from_hours(12.0)
            } else {
                Duration::from_hours(24.0)
            },
            nav: self.matches.get_flag("nav"),
            no_obs: self.matches.get_flag("no-obs"),
        }
    }

    pub fn obs_settings(&self) -> ObsSettings {
        ObsSettings {
            cp: self.matches.get_flag("cp") || self.matches.get_flag("all-meas"),
            pr: self.matches.get_flag("pr") || self.matches.get_flag("all-meas"),
            dop: self.matches.get_flag("dop") || self.matches.get_flag("all-meas"),
            timescale: self.timescale(),
            observables: self.observables(),
            crinex: self.matches.get_flag("crx"),
        }
    }

    pub fn nav_settings(&self) -> NavSettings {
        NavSettings {
            frame_period: Duration::from_hours(2.0),
        }
    }
}
