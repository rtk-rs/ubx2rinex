#![doc(html_logo_url = "https://raw.githubusercontent.com/rtk-rs/.github/master/logos/logo2.jpg")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

/*
 * UBX2RINEX is part of the rtk-rs framework.
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al,
 * (cf. https://github.com/rtk-rs/rinex/graphs/contributors)
 * (cf. https://github.com/rtk-rs/ubx2rinex/graphs/contributors)
 * This framework is shipped under Mozilla Public V2 license.
 *
 * Documentation: https://github.com/rtk-rs/ubx2rinex
 */

extern crate gnss_rs as gnss;
extern crate ublox;

use thiserror::Error;

use rinex::{
    observation::HeaderFields as ObsHeader,
    prelude::{Constellation, Duration, Epoch, Header, Observable, TimeScale, CRINEX, SV},
};

use env_logger::{Builder, Target};
use log::{debug, error, info, trace, warn};

use tokio::sync::mpsc;

use ublox::{GpsFix, NavStatusFlags, NavStatusFlags2, NavTimeUtcFlags, PacketRef, RecStatFlags};

mod cli;
mod collecter;
mod device;
mod utils;

use cli::Cli;
use collecter::{rawxm::Rawxm, Collecter, Message};
use device::Device;

use utils::{gnss_id_to_constellation, to_timescale};

#[derive(Clone)]
pub struct UbloxSettings {
    timescale: TimeScale,
    sampling_period: Duration,
    solutions_ratio: u32,
    constellations: Vec<Constellation>,
    observables: Vec<Observable>,
    sn: Option<String>,
    rx_clock: bool,
    model: Option<String>,
    firmware: Option<String>,
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Unknown constellation id: #{0}")]
    UnknownConstellationId(u8),
}

fn identify_constellation(id: u8) -> Result<Constellation, Error> {
    match id {
        0 => Ok(Constellation::GPS),
        1 => Ok(Constellation::Galileo),
        2 => Ok(Constellation::Glonass),
        3 => Ok(Constellation::BeiDou),
        _ => Err(Error::UnknownConstellationId(id)),
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // pretty_env_logger::init();

    let mut builder = Builder::from_default_env();

    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // cli
    let cli = Cli::new();

    // init
    let mut buffer = [0; 8192];
    let mut uptime = Duration::default();

    let mut fix_type = GpsFix::NoFix; // current fix status
    let mut fix_flags = NavStatusFlags::empty(); // current fix flag
    let mut nav_status = NavStatusFlags2::Inactive;

    // UBlox settings
    let port = cli.port();
    let baud_rate = cli.baud_rate().unwrap_or(115_200);
    let ubx_settings = cli.ublox_settings();

    // RINEX settings
    let settings = cli.rinex_settings();

    let (tx, rx) = mpsc::channel(10);

    let mut collecter = Collecter::new(settings, ubx_settings.clone(), rx);

    // Open device
    let mut device = Device::open(port, baud_rate, &mut buffer);

    device.enable_nav_eoe(&mut buffer);
    debug!("UBX-NAV-EOE enabled");

    device.enable_nav_pvt(&mut buffer);
    debug!("UBX-NAV-PVT enabled");

    device.enable_nav_sat(&mut buffer);
    debug!("UBX-NAV-SAT enabled");

    if ubx_settings.rx_clock {
        device.enable_nav_clock(&mut buffer);
        debug!("UBX-NAV-CLK enabled");
    }

    // device.apply_cfg_rate(&mut buffer, measure_rate_ms, nav_solutions_ratio, time_ref);
    // debug!("Measurement rate is {} ({:?})", sampling, time_ref);

    tokio::spawn(async move {
        collecter.run().await;
    });

    let now = Epoch::now().unwrap_or_else(|e| panic!("Failed to determine system time: {}", e));

    info!("{} - program deployed", now);

    loop {
        let _ = device.consume_all_cb(&mut buffer, |packet| {
            match packet {
                PacketRef::CfgNav5(pkt) => {
                    // Dynamic model
                    let _dyn_model = pkt.dyn_model();
                },
                PacketRef::RxmRawx(pkt) => {
                    let tow_nanos = (pkt.rcv_tow() * 1.0E9).round() as u64;
                    let week = pkt.week();
                    let t =
                        Epoch::from_time_of_week(week as u32, tow_nanos, ubx_settings.timescale);

                    let stat = pkt.rec_stat();

                    if stat.intersects(RecStatFlags::CLK_RESET) {
                        error!("{} - clock reset!", t);
                        warn!("{} - declaring phase cycle slip!", t);
                    }

                    for meas in pkt.measurements() {
                        let pr = meas.pr_mes();
                        let _pr_stddev = meas.pr_stdev();

                        let cp = meas.cp_mes();
                        let _cp_stddev = meas.cp_stdev();

                        let dop = meas.do_mes();
                        let _dop_stddev = meas.do_stdev();

                        // let freq_id = meas.freq_id();
                        let gnss_id = meas.gnss_id();
                        let cno = meas.cno();

                        let constell = gnss_id_to_constellation(gnss_id);
                        if constell.is_none() {
                            debug!("unknown constellation: #{}", gnss_id);
                            continue;
                        }

                        let constell = constell.unwrap();

                        let prn = meas.sv_id();
                        let sv = SV::new(constell, prn);

                        let rawxm = Rawxm::new(t, sv, pr, cp, dop, cno);

                        match tx.blocking_send(Message::Measurement(rawxm)) {
                            Ok(_) => {
                                debug!("{}", rawxm);
                            },
                            Err(e) => {
                                error!("{}({}) missed measurement: {}", t, sv, e);
                            },
                        }
                    }
                },
                PacketRef::MonHw(_pkt) => {
                    //let jamming = pkt.jam_ind(); //TODO
                    //antenna problem:
                    // pkt.a_status();
                    //
                },
                PacketRef::NavSat(pkt) => {
                    debug!("nav-sat: {:?}", pkt);
                    for sv in pkt.svs() {
                        let gnss = identify_constellation(sv.gnss_id());
                        if gnss.is_ok() {
                            let _elev = sv.elev();
                            let _azim = sv.azim();
                            let _pr_res = sv.pr_res();
                            let _flags = sv.flags();

                            let _sv = SV {
                                constellation: gnss.unwrap(),
                                prn: sv.sv_id(),
                            };

                            // flags.sv_used()
                            //flags.health();
                            //flags.quality_ind();
                            //flags.differential_correction_available();
                            //flags.ephemeris_available();
                        }
                    }
                },
                PacketRef::NavTimeUTC(pkt) => {
                    if pkt.valid().intersects(NavTimeUtcFlags::VALID_UTC) {
                        // leap seconds already known
                        let e = Epoch::maybe_from_gregorian(
                            pkt.year().into(),
                            pkt.month(),
                            pkt.day(),
                            pkt.hour(),
                            pkt.min(),
                            pkt.sec(),
                            pkt.nanos() as u32,
                            TimeScale::UTC,
                        );
                    }
                },
                PacketRef::NavStatus(pkt) => {
                    //itow = pkt.itow();
                    fix_type = pkt.fix_type();
                    fix_flags = pkt.flags();
                    nav_status = pkt.flags2();
                    uptime = Duration::from_milliseconds(pkt.uptime_ms() as f64);
                    trace!("uptime: {}", uptime);
                },
                PacketRef::NavEoe(pkt) => {
                    let itow = pkt.itow();
                    // reset Epoch
                    // lli = None;
                    // epoch_flag = EpochFlag::default();
                    debug!("EOE | itow = {}", itow);
                },
                PacketRef::NavPvt(pkt) => {
                    debug!("NAV PVT: {:?}", pkt);
                },
                PacketRef::MgaGpsEph(pkt) => {
                    // let _sv = sv!(&format!("G{}", pkt.sv_id()));
                    //nav_record.insert(epoch, sv);
                },
                PacketRef::MgaGloEph(pkt) => {
                    // let _sv = sv!(&format!("R{}", pkt.sv_id()));
                    //nav_record.insert(epoch, sv);
                },
                /*
                 * NAVIGATION: IONOSPHERIC MODELS
                 */
                PacketRef::MgaGpsIono(pkt) => {
                    // let kbmodel = KbModel {
                    //     alpha: (pkt.alpha0(), pkt.alpha1(), pkt.alpha2(), pkt.alpha3()),
                    //     beta: (pkt.beta0(), pkt.beta1(), pkt.beta2(), pkt.beta3()),
                    //     region: KbRegionCode::default(), // TODO,
                    // };
                    // let _iono = IonMessage::KlobucharModel(kbmodel);
                },
                PacketRef::NavClock(pkt) => {
                    let _bias = pkt.clk_b();
                    let _drift = pkt.clk_d();
                    // pkt.t_acc(); // phase accuracy
                    // pkt.f_acc(); // frequency accuracy
                },
                /*
                 * Errors, Warnings
                 */
                PacketRef::InfTest(pkt) => {
                    if let Some(msg) = pkt.message() {
                        trace!("{}", msg);
                    }
                },
                PacketRef::InfDebug(pkt) => {
                    if let Some(msg) = pkt.message() {
                        debug!("{}", msg);
                    }
                },
                PacketRef::InfNotice(pkt) => {
                    if let Some(msg) = pkt.message() {
                        info!("{}", msg);
                    }
                },
                PacketRef::InfError(pkt) => {
                    if let Some(msg) = pkt.message() {
                        error!("{}", msg);
                    }
                },
                PacketRef::InfWarning(pkt) => {
                    if let Some(msg) = pkt.message() {
                        warn!("{}", msg);
                    }
                },
                pkt => {
                    warn!("main: {:?}", pkt);
                }, // unused
            }
        });
    } // loop
}

//     let deploy_time =
//         Epoch::now().unwrap_or_else(|e| panic!("Failed to determine system (OS) time: {}", e));

//     // U-Blox setup
//     let rx_clock = cli.rx_clock();
//     let sampling = cli.sampling();
//     let anti_spoofing = cli.anti_spoofing();

//     let profile = match cli.profile() {
//         Some(profile) => profile.to_string(),
//         _ => "portable".to_string(),
//     };

//     let constellations = cli.constellations();
//     let constellation = if constellations.len() == 1 {
//         constellations[0]
//     } else {
//         Constellation::Mixed
//     };

//     // Device configurations
//     device
//         .read_version(&mut buffer, &mut rcvr)
//         .unwrap_or_else(|e| panic!("Failed to read firmware version: {}", e));

//     device
//         .read_gnss(&mut buffer)
//         .unwrap_or_else(|e| panic!("Failed to read GNSS constellations: {}", e));

//     if no_obs_rinex {
//     } else {
//         device.enable_obs_rinex(&mut buffer);
//         header.obs = Some(obs_header);
//         info!("Observation RINEX mode deployed");
//     }

//     let rinex = Rinex::basic_obs().with_header(header);

//     let mut t = deploy_time.to_time_scale(timescale);
//     let mut collecter = Collecter::new(&prefix, t, rinex, crinex, false);
