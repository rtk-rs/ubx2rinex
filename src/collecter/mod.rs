use rinex::{
    navigation::Ephemeris,
    prelude::{Epoch, SV},
};

mod fd;

pub mod ephemeris;
pub mod navigation;
pub mod observation;
pub mod rawxm;
pub mod settings;

use rawxm::Rawxm;

pub enum Message {
    /// [Message::Shutdown] catches Ctrl+C interruptions
    Shutdown,
    /// [Message::EndofEpoch] notification
    EndofEpoch(Epoch),
    /// [Epoch] update
    Timestamp(Epoch),
    /// New clock state [s]
    Clock(f64),
    /// New [Rawxm] measurements
    Measurement(Rawxm),
    /// Firmware version notification
    FirmwareVersion(String),
    /// Ephemeris publication
    Ephemeris((Epoch, SV, Ephemeris)),
}
