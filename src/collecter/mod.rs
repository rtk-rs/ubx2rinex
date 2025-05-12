use log::debug;

// pub mod brdc;
pub mod fd;
pub mod observations;
pub mod runtime;
pub mod settings;

use observations::{
    collecter::Collecter as Obscollector, rawxm::Rawxm, settings::Settings as ObsSettings,
};

// use brdc::{collecter::Collecter as BrdcCollector, settings::Settings as BrdcSettings};

use runtime::Runtime;

use settings::Settings;

use rinex::{
    navigation::Ephemeris,
    prelude::{Epoch, SV},
};

use ublox_lib::RxmSfrbxInterpreted;

use crossbeam_channel::Receiver;

pub enum Message {
    /// [Message::Shutdown] catches Ctrl+C interruptions
    Shutdown,

    /// [Message::EndofEpoch] notification
    EndofEpoch(Epoch),

    /// New clock state [s]
    Clock(f64),

    /// [Rawxm]
    Rawxm(Rawxm),

    /// [RxmSfrbxInterpreted]
    Sfrbx(RxmSfrbxInterpreted),

    /// Ephemeris publication
    Ephemeris((Epoch, SV, Ephemeris)),
}

pub struct Collector {
    rtm: Runtime,
    rx: Receiver<Message>,
    shared_settings: Settings,
    obs_settings: ObsSettings,
}

impl Collector {
    pub fn new(
        shared_settings: Settings,
        obs_settings: ObsSettings,
        rx: Receiver<Message>,
    ) -> Self {
        let rtm = Runtime::new();

        Self {
            rx,
            rtm,
            obs_settings,
            shared_settings,
        }
    }

    pub async fn run(&mut self) {
        debug!("{} - OBS RINEX collector deployed", self.rtm.deploy_time);

        let mut obs_rinex = Obscollector::new(
            &self.rtm,
            self.obs_settings.clone(),
            self.shared_settings.clone(),
            self.rx.clone(),
        );

        tokio::spawn(async move {
            obs_rinex.run().await;
        });
    }
}
