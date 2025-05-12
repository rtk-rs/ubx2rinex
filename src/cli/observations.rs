use clap::{Arg, ArgAction};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OBSERVABLES_ARGS: Vec<Arg> = vec![
        Arg::new("all-meas")
            .long("all-meas")
            .action(ArgAction::SetTrue)
            .required_unless_present_any(["no-obs", "cp", "pr", "dop"])
            .help(
                "Activate all measurements (cp, pr, doppler, cno) at once (at least one required)."
            ),
        Arg::new("cp")
            .long("cp")
            .action(ArgAction::SetTrue)
            .required_unless_present_any(["no-obs", "all-meas", "pr", "dop"])
            .help("Activate carrier phase observation (at least one required)"),
        Arg::new("pr")
            .long("pr")
            .action(ArgAction::SetTrue)
            .required_unless_present_any(["no-obs", "cp", "all-meas", "dop"])
            .help("Activate pseudo range observation (at least one required)"),
        Arg::new("dop")
            .long("dop")
            .action(ArgAction::SetTrue)
            .required_unless_present_any(["no-obs", "cp", "pr", "all-meas"])
            .help("Activate doppler observation (at least one required)"),
    ];
}

lazy_static! {
    pub static ref OBSERVATION_ARGS: Vec<Arg> = vec![
        Arg::new("sampling")
            .short('s')
            .long("sampling")
            .required(false)
            .help("Sampling interval. Default value is 30s (standard low-rate RINEX)."),
        Arg::new("timescale")
            .long("timescale")
            .required(false)
            .help(
                "Timescale to express Observations. Default is GPST. 
Any value is supported here, but GNSS timescales are expected by RINEX standards."
            ),
        Arg::new("crx")
            .long("crx")
            .action(ArgAction::SetTrue)
            .help("Add CRINEX compression. Disabled by default."),
    ];
}
