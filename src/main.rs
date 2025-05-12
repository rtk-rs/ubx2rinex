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
extern crate ublox as ublox_lib;

use std::str::FromStr;

use env_logger::{Builder, Target};

use log::{debug, error, info, trace, warn};

use tokio::{
    signal,
    sync::{mpsc, watch},
};

use rinex::prelude::{Carrier, Constellation, Duration, Epoch, Observable, TimeScale, SV};

use ublox_lib::{NavStatusFlags, NavStatusFlags2, NavTimeUtcFlags, PacketRef, RecStatFlags};

mod cli;
mod collecter;
mod ublox;
mod utils;

use crate::{
    cli::Cli,
    collecter::{observations::rawxm::Rawxm, Collector, Message},
    ublox::{Settings as UbloxSettings, Ublox},
    utils::to_constellation,
};

#[tokio::main]
pub async fn main() {
    // pretty_env_logger::init();
    let mut builder = Builder::from_default_env();

    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // cli
    let cli = Cli::new();

    // Settings
    let port = cli.port();
    let baud_rate = cli.baud_rate().unwrap_or(115_200);
    let shared_settings = cli.settings();
    let obs_settings = cli.obs_settings();
    let mut ubx_settings = cli.ublox_settings();

    // init
    let mut buffer = [0; 8192];
    let mut uptime = Duration::default();

    let mut fix_flags = NavStatusFlags::empty(); // current fix flag
    let mut nav_status = NavStatusFlags2::Inactive;

    let timescale = ubx_settings.timescale;

    // Time
    let mut t_utc = Epoch::now()
        .unwrap_or_else(|e| panic!("Failed to determine system time: {}", e))
        .to_time_scale(TimeScale::UTC);

    let mut nav_utc_week = t_utc.to_time_of_week().0;

    let mut t_gpst = t_utc.to_time_scale(TimeScale::GPST);

    let mut nav_gpst = t_gpst;
    let mut nav_gpst_week = t_gpst.to_time_of_week().0;

    let mut t_gst = t_utc.to_time_scale(TimeScale::GST);

    let mut nav_gst = t_gst;
    let mut nav_gst_week = t_gst.to_time_of_week().0;

    let mut t_bdt = t_utc.to_time_scale(TimeScale::BDT);

    let mut nav_bdt = t_bdt;
    let mut nav_bdt_week = t_bdt.to_time_of_week().0;

    let mut end_of_nav_epoch = false;

    // Tokio
    let (shutdown_tx, shutdown_rx) = watch::channel(true);

    let (tx, rx) = crossbeam_channel::bounded(16);

    let mut collecter = Collector::new(shared_settings, obs_settings.clone(), rx);

    let mut ublox = Ublox::open(port, baud_rate, &mut buffer);

    ublox.configure(&ubx_settings, &mut buffer);

    tokio::spawn(async move {
        collecter.run().await;
    });

    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("Tokio signal handling error: {}", e));

        shutdown_tx
            .send(true)
            .unwrap_or_else(|e| panic!("Tokio: signaling error: {}", e));
    });

    loop {
        let _ = ublox.consume_all_cb(&mut buffer, |packet| {
            match packet {
                PacketRef::MonHw(_) => {
                    // TODO
                },
                PacketRef::CfgNav5(pkt) => {
                    // Dynamic model
                    let _dyn_model = pkt.dyn_model();
                },

                PacketRef::RxmRawx(pkt) => {
                    let gpst_tow_nanos = (pkt.rcv_tow() * 1.0E9).round() as u64;
                    t_gpst = Epoch::from_time_of_week(pkt.week() as u32, gpst_tow_nanos, timescale);

                    let stat = pkt.rec_stat();

                    if stat.intersects(RecStatFlags::CLK_RESET) {
                        error!("{} - clock reset!", t_gpst);
                        warn!("{} - declaring phase cycle slip!", t_gpst);
                    }

                    for meas in pkt.measurements() {
                        let (pr, cp, dop, cno) =
                            (meas.pr_mes(), meas.cp_mes(), meas.do_mes(), meas.cno());

                        let gnss_id = meas.gnss_id();
                        let constell = to_constellation(gnss_id);

                        if constell.is_none() {
                            debug!("unknown constellation: #{}", gnss_id);
                            continue;
                        }

                        let constell = constell.unwrap();
                        let sv = SV::new(constell, meas.sv_id());

                        let t = if obs_settings.timescale == TimeScale::GPST {
                            t_gpst
                        } else {
                            t_gpst.to_time_scale(obs_settings.timescale)
                        };

                        trace!(
                            "{}({}) pr={} cp={} dop={} freq_id={}",
                            t,
                            sv,
                            pr,
                            cp,
                            dop,
                            meas.freq_id()
                        );

                        let rawxm = Rawxm {
                            t,
                            sv,
                            pr,
                            cp,
                            dop,
                            cno,
                            carrier: Carrier::L1, //TODO
                        };

                        match tx.send(Message::Rawxm(rawxm)) {
                            Ok(_) => {
                                debug!("{}", rawxm);
                            },
                            Err(e) => {
                                error!("{}({}) missed measurement: {}", t_gpst, sv, e);
                            },
                        }
                    }
                },

                PacketRef::NavClock(pkt) => {
                    let clock = pkt.clk_bias();
                    match tx.try_send(Message::Clock(clock)) {
                        Ok(_) => {
                            debug!("{}", clock);
                        },
                        Err(e) => {
                            error!("missed clock state: {}", e);
                        },
                    }
                },

                PacketRef::NavSat(pkt) => {
                    for sv in pkt.svs() {
                        let constellation = to_constellation(sv.gnss_id());

                        if constellation.is_none() {
                            continue;
                        }

                        let constellation = constellation.unwrap();

                        let _elev = sv.elev();
                        let _azim = sv.azim();
                        let _pr_res = sv.pr_res();
                        let _flags = sv.flags();

                        let _sv = SV {
                            constellation,
                            prn: sv.sv_id(),
                        };

                        // flags.sv_used()
                        //flags.health();
                        //flags.quality_ind();
                        //flags.differential_correction_available();
                        //flags.ephemeris_available();
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
                    pkt.itow();
                    //itow = pkt.itow();
                    uptime = Duration::from_milliseconds(pkt.uptime_ms() as f64);
                    trace!(
                        "Fix status: {:?} | {:?} | {:?}",
                        pkt.fix_stat(),
                        pkt.flags(),
                        pkt.flags2()
                    );
                    trace!("Uptime: {}", uptime);
                },

                PacketRef::NavEoe(pkt) => {
                    let nav_gpst_itow_nanos = pkt.itow() as u64 * 1_000_000;

                    nav_gpst = Epoch::from_time_of_week(
                        nav_gpst_week,
                        nav_gpst_itow_nanos,
                        TimeScale::GPST,
                    );

                    end_of_nav_epoch = true;
                    debug!("{} - End of Epoch", nav_gpst);
                    // let _ = nav_tx.try_send(Message::EndofEpoch(nav_gpst));
                },

                PacketRef::NavPvt(pkt) => {
                    let (y, m, d) = (pkt.year() as i32, pkt.month(), pkt.day());
                    let (hh, mm, ss) = (pkt.hour(), pkt.min(), pkt.sec());
                    if pkt.valid() > 2 {
                        t_utc = Epoch::from_gregorian(y, m, d, hh, mm, ss, 0, TimeScale::UTC)
                            .to_time_scale(timescale);

                        info!(
                            "{} - nav-pvt: lat={:.5E}° long={:.5E}°",
                            t_utc,
                            pkt.latitude(),
                            pkt.longitude()
                        );
                    }
                },

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
                _ => {},
            }
        });
    } // loop
}
