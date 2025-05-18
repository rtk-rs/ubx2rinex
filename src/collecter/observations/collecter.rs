use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    str::FromStr,
};

use rinex::{
    observation::{ClockObservation, HeaderFields as ObsHeader},
    prelude::{
        obs::{EpochFlag, ObsKey, Observations, SignalObservation},
        Constellation, Duration, Epoch, Header, Observable, RinexType, CRINEX,
    },
};

use crossbeam_channel::Receiver;

use log::{error, info};

use crate::collecter::{
    fd::FileDescriptor, observations::settings::Settings, settings::Settings as SharedSettings,
    Message,
};

pub struct Collecter {
    opts: Settings,
    shared_opts: SharedSettings,
    sampling_period: Duration,
    latest_t: Option<Epoch>,
    buffer: Observations,
    rx: Receiver<Message>,
    time_of_first_obs: Option<Epoch>,
    constellations: Vec<Constellation>,
    header: Option<ObsHeader>,
    header_released: bool,
    fd: Option<BufWriter<FileDescriptor>>,
}

impl Collecter {
    /// Builds new [Collecter]
    pub fn new(
        opts: Settings,
        sampling_period: Duration,
        shared_opts: SharedSettings,
        rx: Receiver<Message>,
    ) -> Self {
        Self {
            rx,
            fd: None,
            opts,
            shared_opts,
            header: None,
            latest_t: None,
            header_released: false,
            time_of_first_obs: None,
            sampling_period,
            buffer: Observations::default(),
            constellations: Vec::with_capacity(4),
        }
    }

    /// Obtain a new file descriptor
    fn fd(&self, t: Epoch) -> FileDescriptor {
        let filename = self.opts.filename(t, &self.shared_opts);
        FileDescriptor::new(self.shared_opts.gzip, &filename)
    }

    pub async fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(msg) => match msg {
                    Message::Shutdown => {
                        // not really handled
                        return;
                    },

                    Message::Clock(clock) => {
                        let bias = clock * 1.0E-3;
                        let mut clock = ClockObservation::default();
                        clock.set_offset_s(Default::default(), bias);
                        self.buffer.clock = Some(clock);
                    },

                    Message::Constellations(constellations) => {
                        self.constellations = constellations.clone();
                    },

                    Message::Rawxm(rawxm) => {
                        if !self.constellations.contains(&rawxm.sv.constellation) {
                            // unknown / unexpected: should not happen
                            continue;
                        }

                        if let Some(latest_t) = self.latest_t {
                            let time_of_first_obs = self.time_of_first_obs.unwrap();
                            let dt = latest_t - time_of_first_obs;

                            if dt >= self.shared_opts.snapshot_period {
                                info!("{} - file publication", rawxm.t);
                                self.time_of_first_obs = None;
                                self.header_released = false;
                            }
                        }

                        if self.time_of_first_obs.is_none() {
                            self.time_of_first_obs = Some(rawxm.t);
                        }

                        if self.latest_t.is_none() {
                            self.latest_t = Some(rawxm.t);
                        }

                        if self.may_release_header() && !self.header_released {
                            self.release_header();
                            self.header_released = true;
                        }

                        let latest_t = self.latest_t.unwrap();

                        if rawxm.t > latest_t {
                            // New epoch
                            if self.has_pending_content() {
                                self.release_epoch();
                            }
                        }

                        if self.opts.pr {
                            let c1c = if self.shared_opts.major > 2 {
                                Observable::from_str("C1C").unwrap()
                            } else {
                                Observable::from_str("C1").unwrap()
                            };

                            self.buffer.signals.push(SignalObservation {
                                sv: rawxm.sv,
                                lli: None, // TODO
                                snr: None, // TODO
                                value: rawxm.pr,
                                observable: c1c,
                            });
                        }

                        if self.opts.cp {
                            let l1c = if self.shared_opts.major > 2 {
                                Observable::from_str("L1C").unwrap()
                            } else {
                                Observable::from_str("L1").unwrap()
                            };

                            self.buffer.signals.push(SignalObservation {
                                sv: rawxm.sv,
                                lli: None, // TODO
                                snr: None, // TODO
                                value: rawxm.cp,
                                observable: l1c,
                            });
                        }

                        if self.opts.dop {
                            let d1c = if self.shared_opts.major > 2 {
                                Observable::from_str("D1C").unwrap()
                            } else {
                                Observable::from_str("D1").unwrap()
                            };

                            self.buffer.signals.push(SignalObservation {
                                sv: rawxm.sv,
                                lli: None, // TODO
                                snr: None, // TODO
                                value: rawxm.dop as f64,
                                observable: d1c,
                            });
                        }

                        // update
                        let mut new_t = rawxm.t;

                        if self.sampling_period < Duration::from_seconds(1.0) {
                            new_t = new_t.round(Duration::from_seconds(1.0));
                        }

                        self.latest_t = Some(new_t);
                    },
                    _ => {},
                },

                Err(e) => {
                    error!("obs_rinex: recv error {}", e);
                    return;
                },
            }
        }
    }

    fn release_header(&mut self) {
        let time_of_first_obs = self.time_of_first_obs.unwrap();

        // obtain new file, release header
        let mut fd = BufWriter::new(self.fd(time_of_first_obs));

        let header = self.build_header();

        header.format(&mut fd).unwrap_or_else(|e| {
            panic!(
                "RINEX header formatting: {}. Aborting (avoiding corrupt file)",
                e
            )
        });

        let _ = fd.flush();

        self.fd = Some(fd);
        self.header = Some(header.obs.unwrap().clone());
    }

    fn release_epoch(&mut self) {
        let latest_t = self.latest_t.unwrap();

        let key = ObsKey {
            epoch: latest_t,
            flag: EpochFlag::Ok, // TODO,
        };

        let mut fd = self.fd.as_mut().unwrap();

        let header = self
            .header
            .as_ref()
            .expect("internal error: missing Observation header");

        match self
            .buffer
            .format(self.shared_opts.major == 2, &key, header, &mut fd)
        {
            Ok(_) => {
                // clear for next time
                let _ = fd.flush();
                self.buffer.clock = None;
                self.buffer.signals.clear();
            },
            Err(e) => {
                error!("{} formatting issue: {}", latest_t, e);
            },
        }
    }

    fn build_header(&self) -> Header {
        let mut header = Header::default();

        header.rinex_type = RinexType::ObservationData;
        header.version.major = self.shared_opts.major;

        let mut obs_header = ObsHeader::default();

        if self.opts.crinex {
            let mut crinex = CRINEX::default();

            if self.shared_opts.major == 2 {
                crinex.version.major = 2;
            } else {
                crinex.version.major = 3;
            }

            obs_header.crinex = Some(crinex);
        }

        if let Some(operator) = &self.shared_opts.operator {
            header.observer = Some(operator.clone());
        }

        if let Some(agency) = &self.shared_opts.agency {
            header.agency = Some(agency.clone());
        }

        let mut observables = HashMap::<Constellation, Vec<Observable>>::new();

        for constellation in self.constellations.iter() {
            let mut codes = Vec::new();

            if self.opts.pr {
                if self.shared_opts.major > 2 {
                    codes.push("C1C");
                } else {
                    codes.push("C1");
                }
            }

            if self.opts.cp {
                if self.shared_opts.major > 2 {
                    codes.push("L1C");
                } else {
                    codes.push("L1");
                }
            }

            if self.opts.dop {
                if self.shared_opts.major > 2 {
                    codes.push("D1C");
                } else {
                    codes.push("D1");
                }
            }

            observables.insert(
                *constellation,
                codes
                    .iter()
                    .map(|code| Observable::from_str(code).unwrap())
                    .collect(),
            );
        }

        obs_header.codes = observables;

        header.obs = Some(obs_header);
        header
    }

    fn may_release_header(&self) -> bool {
        !self.constellations.is_empty() && self.time_of_first_obs.is_some()
    }

    fn has_pending_content(&self) -> bool {
        self.buffer.signals.len() > 0 || self.buffer.clock.is_some()
    }
}
