use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
};

use log::{error, info};

use rinex::{
    navigation::{NavFrame, NavFrameType, NavKey, NavMessageType},
    prelude::{Epoch, Header, Version},
    record::Record,
};

use tokio::{sync::mpsc::Receiver as Rx, sync::watch::Receiver as WatchRx};

use crate::{
    collecter::{
        brdc::{ephemeris::EphBuffer, settings::Settings},
        fd::FileDescriptor,
        runtime::Runtime,
        settings::Settings as SharedSettings,
        Message,
    },
    ublox::Settings as UbloxSettings,
};

pub struct Collecter {
    rx: Rx<Message>,
    deploy_time: Epoch,
    first_t: Option<Epoch>,
    opts: Settings,
    shared_opts: SharedSettings,
    header_released: bool,
    header: Option<Header>,
    fd: Option<BufWriter<FileDescriptor>>,
    eph_buffer: EphBuffer,
}

impl Collecter {
    /// Builds new [Collecter]
    pub fn new(
        rtm: &Runtime,
        opts: Settings,
        shared_opts: SharedSettings,
        rx: Rx<Message>,
    ) -> Self {
        Self {
            rx,
            fd: None,
            first_t: None,
            header_released: false,
            eph_buffer: EphBuffer::new(),
            deploy_time: rtm.deploy_time,
            opts,
            shared_opts,
            header: None,
        }
    }

    /// Obtain a new file descriptor
    fn fd(&self, t: Epoch) -> FileDescriptor {
        let filename = self.opts.filename(t, &self.shared_opts);
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
            match self.rx.recv().await {
                Some(msg) => match msg {
                    Message::EndofEpoch(t) => {},

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
                        let header = self.header.as_ref().unwrap();

                        // for eph in eph_buffer.iter().filter(|eph| eph.is_ready()) {
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
                        // }
                    },

                    _ => {},
                },
                None => {},
            }
        }
    }

    fn may_release_header(&self) -> bool {
        self.first_t.is_some()
    }

    fn release_header(&mut self) {

        // let version = Version::new(shared_opts.major, 0);

        // let mut header = Header::basic_nav().with_version(version);

        // if let Some(agency) = &shared_opts.agency {
        //     header.agency = Some(agency.to_string());
        // }

        // // obtain a file descriptor
        // let mut fd = BufWriter::new(self.fd());

        // self.header.format(&mut fd).unwrap_or_else(|e| {
        //     panic!(
        //         "RINEX header formatting: {}. Aborting (avoiding corrupt file)",
        //         e
        //     );
        // });

        // let _ = fd.flush();
        // self.fd = Some(fd);
    }
}
