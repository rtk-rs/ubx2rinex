use clap::{Arg, ArgAction};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NAVIGATION_ARGS: Vec<Arg> = vec![Arg::new("no-eph")
        .long("no-eph")
        .action(ArgAction::SetTrue)
        .help("Disable Ephemeris collection (enabled by default)."),];
}
