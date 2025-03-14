use std::{
    io::{BufWriter, Write},
    str::FromStr,
};

use rinex::{
    observation::ClockObservation,
    prelude::{
        //Carrier,
        //Constellation,
        Duration,
        Epoch,
        //TimeScale,
        CRINEX,
    },
};

use tokio::sync::mpsc::Receiver;

// use itertools::Itertools;

use log::error;

use rinex::{
    observation::HeaderFields as ObsHeader,
    prelude::{
        obs::{EpochFlag, ObsKey, Observations, SignalObservation},
        Header, Observable,
    },
};

use crate::UbloxSettings;

mod fd;

pub mod rawxm;
pub mod settings;

use fd::FileDescriptor;
use rawxm::Rawxm;
use settings::Settings;

// #[derive(Debug, Default, Copy, Clone)]
// enum State {
//     #[default]
//     FirmwareVersion,
//     Constellations,
//     Observables,
//     Header,
//     Collecting,
//     Release,
// }

#[derive(Debug)]
pub enum Message {
    Shutdown,
    Clock(f64),
    Measurement(Rawxm),
    FirmwareVersion(String),
}

pub struct Collecter {
    t: Option<Epoch>,
    t0: Option<Epoch>,
    buf: Observations,
    obs_header: Option<ObsHeader>,
    rx: Receiver<Message>,
    shutdown: bool,
    settings: Settings,
    ubx_settings: UbloxSettings,
    fd: Option<BufWriter<FileDescriptor>>,
}

impl Collecter {
    /// Builds new [Collecter]
    pub fn new(settings: Settings, ublox: UbloxSettings, rx: Receiver<Message>) -> Self {
        Self {
            rx,
            fd: None,
            t0: None,
            t: None,
            settings,
            shutdown: false,
            obs_header: None,
            ubx_settings: ublox,
            buf: Observations::default(),
        }
    }

    /// Obtain a new file descriptor
    fn fd(&self, t: Epoch) -> FileDescriptor {
        let filename = self.settings.filename(t);
        FileDescriptor::new(self.settings.gzip, &filename)
    }

    pub async fn run(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(msg) => match msg {
                    Message::FirmwareVersion(version) => {
                        self.ubx_settings.firmware = Some(version.to_string());
                    },

                    Message::Shutdown => {
                        if self.buf.signals.len() > 0 || self.buf.clock.is_some() {
                            self.release_epoch();
                        }
                        return;
                    },

                    Message::Clock(clock) => {
                        let bias = clock * 1.0E-3;
                        let mut clock = ClockObservation::default();
                        clock.set_offset_s(Default::default(), bias);
                        self.buf.clock = Some(clock);
                    },

                    Message::Measurement(rawxm) => {
                        if self.t0.is_none() {
                            self.t0 = Some(rawxm.t);
                            self.release_header();
                        }

                        if self.t.is_none() {
                            self.t = Some(rawxm.t);
                        }

                        let t = self.t.unwrap();

                        if rawxm.t > t {
                            if self.buf.signals.len() > 0 || self.buf.clock.is_some() {
                                self.release_epoch();
                            }
                        }

                        let c1c = if self.settings.major == 3 {
                            Observable::from_str("C1C").unwrap()
                        } else {
                            Observable::from_str("C1").unwrap()
                        };

                        let l1c = if self.settings.major == 3 {
                            Observable::from_str("L1C").unwrap()
                        } else {
                            Observable::from_str("L1").unwrap()
                        };

                        let d1c = if self.settings.major == 3 {
                            Observable::from_str("D1C").unwrap()
                        } else {
                            Observable::from_str("D1").unwrap()
                        };

                        self.buf.signals.push(SignalObservation {
                            sv: rawxm.sv,
                            lli: None,
                            snr: None,
                            value: rawxm.cp,
                            observable: c1c,
                        });

                        self.buf.signals.push(SignalObservation {
                            sv: rawxm.sv,
                            lli: None,
                            snr: None,
                            value: rawxm.pr,
                            observable: l1c,
                        });

                        self.buf.signals.push(SignalObservation {
                            sv: rawxm.sv,
                            lli: None,
                            snr: None,
                            value: rawxm.dop as f64,
                            observable: d1c,
                        });

                        self.t = Some(rawxm.t);
                    },
                },
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                },
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
    }

    fn release_header(&mut self) {
        let t0 = self.t0.unwrap();

        // obtain new file, release header
        let mut fd = BufWriter::new(self.fd(t0));

        let header = self.build_header();

        header.format(&mut fd).unwrap_or_else(|e| {
            panic!(
                "RINEX header formatting: {}. Aborting (avoiding corrupt file)",
                e
            )
        });

        let _ = fd.flush();

        self.fd = Some(fd);
        self.obs_header = Some(header.obs.unwrap().clone());
    }

    fn release_epoch(&mut self) {
        let t = self.t.unwrap();

        let key = ObsKey {
            epoch: t,
            flag: EpochFlag::Ok, // TODO,
        };

        let mut fd = self.fd.as_mut().unwrap();

        let obs_header = self
            .obs_header
            .as_ref()
            .expect("internal error: missing Observation header");

        match self
            .buf
            .format(self.settings.major == 2, &key, obs_header, &mut fd)
        {
            Ok(_) => {
                let _ = fd.flush();
                self.buf.clock = None;
                self.buf.signals.clear();
            },
            Err(e) => {
                error!("{} formatting issue: {}", t, e);
            },
        }
    }

    fn build_header(&self) -> Header {
        let mut header = Header::default();
        let mut obs_header = ObsHeader::default();

        header.version.major = self.settings.major;

        if self.settings.crinex {
            let mut crinex = CRINEX::default();

            if self.settings.major == 2 {
                crinex.version.major = 2;
            } else {
                crinex.version.major = 3;
            }

            obs_header.crinex = Some(crinex);
        }

        if let Some(operator) = &self.settings.operator {
            header.observer = Some(operator.clone());
        }

        if let Some(agency) = &self.settings.agency {
            header.agency = Some(agency.clone());
        }

        for constellation in self.ubx_settings.constellations.iter() {
            for observable in self.ubx_settings.observables.iter() {
                if let Some(codes) = obs_header.codes.get_mut(constellation) {
                    codes.push(observable.clone());
                } else {
                    obs_header
                        .codes
                        .insert(*constellation, vec![observable.clone()]);
                }
            }
        }

        header.obs = Some(obs_header);
        header
    }

    /// Returns collecter uptime (whole session)
    fn uptime(&self, t: Epoch) -> Option<Duration> {
        let t0 = self.t0?;
        Some(t - t0)
    }
}
