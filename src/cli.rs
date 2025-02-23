use clap::{Arg, ArgAction, ArgMatches, ColorChoice, Command};

use rinex::prelude::{Constellation, Duration};

pub struct Cli {
    /// Arguments passed by user
    matches: ArgMatches,
}

impl Cli {
    /// Build new command line interface
    pub fn new() -> Self {
        Self {
            matches: {
                Command::new("ubx2rinex")
                    .author("Guillaume W. Bres, <guillaume.bressaix@gmail.com>")
                    .version(env!("CARGO_PKG_VERSION"))
                    .about("U-Blox stream to RINEX collecter")
                    .color(ColorChoice::Always)
                    .arg_required_else_help(true)
                    .next_help_heading("Serial port")
                    .arg(
                        Arg::new("port")
                            .short('p')
                            .long("port")
                            .value_name("PORT")
                            .required(true)
                            .help("Define serial port, on your system. Example /dev/ttyUSB0 on Linux")
                    )
                    .arg(
                        Arg::new("baudrate")
                            .short('b')
                            .long("baud")
                            .required(false)
                            .value_name("Baudrate (u32)")
                            .help("Define serial port baud rate. Communications will not work if your U-Blox streams at a different data-rate. By default we use 115_200"),
                    )
                    .next_help_heading("U-Blox configuration")
                    .arg(
                        Arg::new("gps")
                            .long("gps")
                            .action(ArgAction::SetTrue)
                            .help("Activate GPS constellation")
                            .required_unless_present_any(["galileo", "beidou", "qzss", "glonass"]),
                    )
                    .arg(
                        Arg::new("galileo")
                            .long("galileo")
                            .action(ArgAction::SetTrue)
                            .help("Activate Galileo constellation")
                            .required_unless_present_any(["gps", "beidou", "qzss", "glonass"]),
                    )
                    .arg(
                        Arg::new("bds")
                            .long("bds")
                            .action(ArgAction::SetTrue)
                            .help("Activate BDS (BeiDou) constellation")
                            .required_unless_present_any(["galileo", "gps", "qzss", "glonass"]),
                    )
                    .arg(
                        Arg::new("qzss")
                            .long("qzss")
                            .action(ArgAction::SetTrue)
                            .help("Activate QZSS constellation")
                            .required_unless_present_any(["galileo", "gps", "bds", "glonass"]),
                    )
                    .arg(
                        Arg::new("glonass")
                            .long("glonass")
                            .action(ArgAction::SetTrue)
                            .help("Activate Glonass constellation")
                            .required_unless_present_any(["galileo", "gps", "bds", "qzss"]),
                    )
                    .arg(
                        Arg::new("profile")
                            .long("prof")
                            .action(ArgAction::Set)
                            .help("Define user profile. Default is set to portable!"),
                    )
                    .arg(
                        Arg::new("nav-clock")
                            .long("nav-clock")
                            .action(ArgAction::SetTrue)
                            .help("Resolve local clock state. Disabled by default"),
                    )
                    .arg(
                        Arg::new("anti-spoofing")
                            .long("anti-spoofing")
                            .action(ArgAction::SetTrue)
                            .help("Makes sure anti jamming/spoofing is enabled. When enabled, it is automatically emphasized in the collected RINEX."))
                    .next_help_heading("RINEX Collection")
                    .arg(
                        Arg::new("nav")
                            .long("nav")
                            .action(ArgAction::SetTrue)
                            .help("Activate Navigation RINEX collection. Use this to collect NAV RINEX file(s). File type is closely tied to enabled Constellation(s)."),
                    )
                    .arg(
                        Arg::new("no-obs")
                            .long("no-obs")
                            .action(ArgAction::SetTrue)
                            .help("Disable Observation RINEX collection. You can use this if you intend to collect Ephemerides only for example"),
                    )
                    .next_help_heading("Observation collection (signal sampling)")
                    .arg(
                        Arg::new("sampling")
                            .short('s')
                            .long("sampling")
                            .required(false)
                            .help("Define the sampling interval. Default value is 30s (standard low-rate RINEX).")
                    )
                    .get_matches()
            },
        }
    }

    pub fn constellations(&self) -> Vec<Constellation> {
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
        if self.glonass() {
            constellations.push(Constellation::Glonass);
        }
        constellations
    }

    pub fn nav_clock(&self) -> bool {
        self.matches.get_flag("nav-clock")
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

    pub fn gps(&self) -> bool {
        self.matches.get_flag("gps")
    }
    pub fn galileo(&self) -> bool {
        self.matches.get_flag("galileo")
    }
    pub fn bds(&self) -> bool {
        self.matches.get_flag("bds")
    }
    pub fn qzss(&self) -> bool {
        self.matches.get_flag("qzss")
    }
    pub fn glonass(&self) -> bool {
        self.matches.get_flag("glonass")
    }

    pub fn no_obs_rinex(&self) -> bool {
        self.matches.get_flag("no-obs")
    }

    pub fn nav_rinex(&self) -> bool {
        self.matches.get_flag("nav")
    }

    pub fn anti_spoofing(&self) -> bool {
        self.matches.get_flag("anti-spoofing")
    }

    pub fn profile(&self) -> Option<&String> {
        self.matches.get_one::<String>("profile")
    }

    pub fn sampling(&self) -> Duration {
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
}
