use log::debug;

pub mod brdc;
pub mod fd;
pub mod observations;
pub mod runtime;
pub mod settings;

use observations::{
    collecter::Collecter as Obscollector, rawxm::Rawxm, settings::Settings as ObsSettings,
};

use brdc::{collecter::Collecter, settings::Settings as NavSettings, sfrbx::Sfrbx};

use runtime::Runtime;

use settings::Settings;

use rinex::{
    navigation::Ephemeris,
    prelude::{Constellation, Duration, Epoch, SV},
};

use crossbeam_channel::Receiver;

pub enum Message {
    /// [Message::Shutdown] catches Ctrl+C interruptions
    Shutdown,

    /// [Message::EndofEpoch] notification
    EndofEpoch(Epoch),

    /// Constellations
    Constellations(Vec<Constellation>),

    /// New clock state [s]
    Clock(f64),

    /// [Rawxm]
    Rawxm(Rawxm),

    /// [Sfrbx]
    Sfrbx(Sfrbx),

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

    pub async fn run(&mut self, sampling_period: Duration) {
        if !self.shared_settings.no_obs {
            debug!("{} - OBS RINEX collecter deployed", self.rtm.deploy_time);

            let mut obs_rinex = Obscollector::new(
                &self.rtm,
                self.obs_settings.clone(),
                sampling_period,
                self.shared_settings.clone(),
                self.rx.clone(),
            );

            tokio::spawn(async move {
                obs_rinex.run().await;
            });
        }

        if self.shared_settings.nav {
            debug!("{} - NAV RINEX collecter deployed", self.rtm.deploy_time);

            tokio::spawn(async move {});
        }
    }
}
