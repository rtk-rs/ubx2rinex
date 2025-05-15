use std::io::{BufWriter, Write};

use rinex::prelude::{Constellation, Epoch, Header, Version};

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
            opts,
            shared_opts,
            header_released: false,
            deploy_time: rtm.deploy_time,
            constellations: Default::default(),
        }
    }

    /// Obtain a new file descriptor
    fn fd(&self, t: Epoch) -> FileDescriptor {
        let filename = self
            .opts
            .filename(t, &self.constellations, &self.shared_opts);
        FileDescriptor::new(self.shared_opts.gzip, &filename)
    }

    // fn collect_ephemeris_content(&self, buffer: &EphBuffer) -> Entry {}

    // fn toc(week: u16, toc: u32) -> Epoch {
    //     let toc_nanos = (toc as f64) * 1_000_000_000;
    //     Epoch::from_time_of_week(week, toc_nanos)
    // }

    pub async fn run(&mut self) {
        let mut eph_buffer = EphBuffer::new();

        loop {
            match self.rx.recv() {
                Ok(msg) => match msg {
                    Message::EndofEpoch(t) => {},

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

                        let fd = self.fd.as_mut().unwrap();

                        for eph in eph_buffer.buffer.iter().filter(|eph| eph.is_complete()) {
                            //     if let Some(toc) = eph_buffer.toc() {

                            // if let Some(latest_toc) = self.latest_toc.get_mut(&eph.sv) {
                            //     if toc - latest_toc >= Duration::from_hours(2.0) {
                            //         let rinex = eph.to_rinex();

                            //         match rinex.format(fd, header) {
                            //             Ok(_) => {
                            //                 info!("{} - released NAV content", latest_t);
                            //             },
                            //             Err(e) => {
                            //                 error!("{} - failed to release NAV content: {}", latest_t, e);
                            //             },
                            //         }

                            //         *latest_toc = toc;
                            //     }
                            // }
                            //     }
                        }
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
