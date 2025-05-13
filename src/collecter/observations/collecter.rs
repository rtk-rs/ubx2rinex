use std::{
    io::{BufWriter, Write},
    str::FromStr,
};

use rinex::{
    observation::{ClockObservation, HeaderFields as ObsHeader},
    prelude::{
        obs::{EpochFlag, ObsKey, Observations, SignalObservation},
        Epoch, Header, Observable, RinexType, CRINEX,
    },
};

use crossbeam_channel::Receiver;

use log::error;

use crate::{
    collecter::{
        fd::FileDescriptor, observations::settings::Settings, runtime::Runtime,
        settings::Settings as SharedSettings, Message,
    },
};

pub struct Collecter {
    opts: Settings,
    shared_opts: SharedSettings,
    deploy_time: Epoch,
    latest_t: Option<Epoch>,
    buffer: Observations,
    rx: Receiver<Message>,
    time_of_first_obs: Option<Epoch>,
    header: Option<ObsHeader>,
    fd: Option<BufWriter<FileDescriptor>>,
}

impl Collecter {
    /// Builds new [Collecter]
    pub fn new(
        rtm: &Runtime,
        opts: Settings,
        shared_opts: SharedSettings,
        rx: Receiver<Message>,
    ) -> Self {
        let deploy_time = rtm.deploy_time;

        Self {
            rx,
            deploy_time,
            fd: None,
            opts,
            shared_opts,
            header: None,
            latest_t: None,
            time_of_first_obs: None,
            buffer: Observations::default(),
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

                    Message::Rawxm(rawxm) => {
                        if self.time_of_first_obs.is_none() {
                            self.time_of_first_obs = Some(rawxm.t);
                            self.release_header();
                        }

                        if self.latest_t.is_none() {
                            self.latest_t = Some(rawxm.t);
                        }

                        let latest_t = self.latest_t.unwrap();

                        if rawxm.t > latest_t {
                            // New epoch
                            if self.has_pending_content() {
                                self.release_epoch();
                            }
                        }

                        let c1c = if self.shared_opts.major == 3 {
                            Observable::from_str("C1C").unwrap()
                        } else {
                            Observable::from_str("C1").unwrap()
                        };

                        let l1c = if self.shared_opts.major == 3 {
                            Observable::from_str("L1C").unwrap()
                        } else {
                            Observable::from_str("L1").unwrap()
                        };

                        let d1c = if self.shared_opts.major == 3 {
                            Observable::from_str("D1C").unwrap()
                        } else {
                            Observable::from_str("D1").unwrap()
                        };

                        self.buffer.signals.push(SignalObservation {
                            sv: rawxm.sv,
                            lli: None, // TODO
                            snr: None, // TODO
                            value: rawxm.cp,
                            observable: c1c,
                        });

                        self.buffer.signals.push(SignalObservation {
                            sv: rawxm.sv,
                            lli: None, // TODO
                            snr: None, // TODO
                            value: rawxm.pr,
                            observable: l1c,
                        });

                        self.buffer.signals.push(SignalObservation {
                            sv: rawxm.sv,
                            lli: None, // TODO
                            snr: None, // TODO
                            value: rawxm.dop as f64,
                            observable: d1c,
                        });

                        // update
                        self.latest_t = Some(rawxm.t);
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

        obs_header.codes = self.opts.observables.clone();

        header.obs = Some(obs_header);
        header
    }

    fn has_pending_content(&self) -> bool {
        self.buffer.signals.len() > 0 || self.buffer.clock.is_some()
    }
}
