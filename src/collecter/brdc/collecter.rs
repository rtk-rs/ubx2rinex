use std::io::{BufWriter, Write};

use rinex::{
    navigation::{NavFrame, NavFrameType, NavKey, NavMessageType},
    prelude::{Constellation, Duration, Epoch, Header, Version},
};

use log::error;

use crossbeam_channel::Receiver;

use crate::collecter::{
    brdc::{ephemeris::EphBuffer, settings::Settings},
    fd::FileDescriptor,
    runtime::Runtime,
    settings::Settings as SharedSettings,
    Message,
};
pub struct Collecter {
    rx: Receiver<Message>,
    deploy_time: Epoch,
    first_t: Option<Epoch>,
    opts: Settings,
    next_publication: Epoch,
    shared_opts: SharedSettings,
    header_released: bool,
    constellations: Vec<Constellation>,
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
        Self {
            rx,
            fd: None,
            first_t: None,
            shared_opts,
            header_released: false,
            next_publication: {
                let midnight = rtm.deploy_time.round(Duration::from_days(1.0));
                let mut next = midnight;
                loop {
                    if next > rtm.deploy_time + opts.frame_period {
                        break;
                    }
                    next += opts.frame_period;
                }
                next
            },
            deploy_time: rtm.deploy_time,
            constellations: Default::default(),
            opts,
        }
    }

    /// Obtain a new file descriptor
    fn fd(&self, t: Epoch) -> FileDescriptor {
        let filename = self
            .opts
            .filename(t, &self.constellations, &self.shared_opts);
        FileDescriptor::new(self.shared_opts.gzip, &filename)
    }

    pub async fn run(&mut self) {
        let mut latest_t = self.deploy_time;
        let mut eph_buffer = EphBuffer::new();

        loop {
            match self.rx.recv() {
                Ok(msg) => match msg {
                    Message::Constellations(constellations) => {
                        self.constellations = constellations.clone();
                    },

                    Message::Sfrbx(sfrbx) => {
                        eph_buffer.latch_rxm_sfrbx(sfrbx);

                        if !self.header_released && self.may_release_header() {
                            self.release_header();
                            self.header_released = true;
                        }

                        if !self.header_released {
                            continue;
                        }

                        let mut min_dt = Duration::ZERO;
                        let fd = self.fd.as_mut().unwrap();

                        for eph in eph_buffer.buffer.iter().filter(|eph| eph.is_complete()) {
                            if let Some(toc) = eph.toc(latest_t) {
                                if toc >= self.next_publication {
                                    if let Some(rinex) = eph.to_rinex() {
                                        let key = NavKey {
                                            epoch: toc,
                                            sv: eph.sv,
                                            msgtype: rinex::navigation::NavMessageType::LNAV,
                                            frmtype: rinex::navigation::NavFrameType::Ephemeris,
                                        };
                                        let frame = NavFrame::EPH(rinex);
                                    }
                                } else {
                                    let dt = toc - self.next_publication;
                                }

                                if toc > latest_t {
                                    latest_t = toc;
                                }
                            }
                        }

                        if min_dt > Duration::ZERO {}
                    },

                    _ => {},
                },
                Err(e) => {
                    error!("recv error: {}", e);
                    return;
                },
            }
        }
    }

    fn may_release_header(&self) -> bool {
        self.first_t.is_some()
    }

    fn release_header(&mut self) {
        let version = Version::new(self.shared_opts.major, 0);

        let mut header = Header::basic_nav().with_version(version);

        if let Some(agency) = &self.shared_opts.agency {
            header.agency = Some(agency.to_string());
        }

        // obtain a file descriptor
        // TODO: use first message toc
        let mut fd = BufWriter::new(self.fd(self.deploy_time));

        header.format(&mut fd).unwrap_or_else(|e| {
            panic!(
                "RINEX header formatting: {}. Aborting (avoiding corrupt file)",
                e
            );
        });

        let _ = fd.flush();
        self.fd = Some(fd);
    }
}
