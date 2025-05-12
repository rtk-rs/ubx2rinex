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
    collecter::{brdc::Settings as BrdcSettings, fd::FileDescriptor, settings::Settings, Message},
    ublox::Settings as UbloxSettings,
};

pub struct Collecter {
    /// T0: initial deploy time, updated on each file release.
    t0: Epoch,
    rx: Rx<Message>,
    shutdown: WatchRx<bool>,
    settings: Settings,
    header: Header,
    record: Record,
    ubx_settings: UbloxSettings,
    fd: Option<BufWriter<FileDescriptor>>,
}

impl Collecter {
    /// Builds new [Collecter]
    pub fn new(
        t0: Epoch,
        settings: Settings,
        ublox: UbloxSettings,
        shutdown: WatchRx<bool>,
        rx: Rx<Message>,
    ) -> Self {
        let version = Version::new(settings.major, 0);

        let mut header = Header::basic_nav().with_version(version);

        if let Some(operator) = &settings.operator {
            header.observer = Some(operator.clone());
        }

        if let Some(agency) = &settings.agency {
            header.agency = Some(agency.to_string());
        }

        Self {
            t0,
            rx,
            settings,
            header,
            fd: None,
            shutdown,
            ubx_settings: ublox,
            record: Record::NavRecord(BTreeMap::new()),
        }
    }

    /// Obtain a new file descriptor
    fn fd(&self) -> FileDescriptor {
        let t = self.t0;
        let filename = self.settings.filename(true, t);
        FileDescriptor::new(self.settings.gzip, &filename)
    }

    pub async fn run(&mut self) {
        loop {
            match self.rx.recv().await {
                Some(msg) => match msg {
                    Message::EndofEpoch(t) => {
                        if self.fd.is_none() {
                            self.release_header();
                        }

                        let fd = self.fd.as_mut().unwrap();

                        match self.record.format(fd, &self.header) {
                            Ok(_) => {
                                info!("{} - released new epoch", t);
                            },
                            Err(e) => {
                                error!("{} - RINEX formatting error: {}", t, e);
                            },
                        }
                    },

                    Message::FirmwareVersion(version) => {
                        self.ubx_settings.firmware = Some(version.to_string());
                    },

                    Message::Ephemeris((t, sv, eph)) => {
                        let key = NavKey {
                            epoch: t,
                            sv,
                            msgtype: NavMessageType::LNAV,
                            frmtype: NavFrameType::Ephemeris,
                        };

                        let frame = NavFrame::EPH(eph);

                        let rec = self
                            .record
                            .as_mut_nav()
                            .expect("internal error: invalid nav setup");

                        rec.insert(key, frame);
                    },

                    Message::Shutdown => {
                        return;
                    },

                    _ => {},
                },
                None => {},
            }
        }
    }

    fn release_header(&mut self) {
        // obtain a file descriptor
        let mut fd = BufWriter::new(self.fd());

        self.header.format(&mut fd).unwrap_or_else(|e| {
            panic!(
                "RINEX header formatting: {}. Aborting (avoiding corrupt file)",
                e
            )
        });

        let _ = fd.flush();
        self.fd = Some(fd);
    }
}
