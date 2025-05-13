use rinex::prelude::{Constellation, Duration, TimeScale};
use ublox_lib::{cfg_val::CfgVal, CfgLayerSet, CfgValSetBuilder};

#[derive(Debug, Clone)]
pub struct Settings {
    /// L1 activated for all constellations
    pub l1: bool,
    /// L2 activated for all constellations
    pub l2: bool,
    /// L5 activated for all constellations
    pub l5: bool,
    /// Timescale we align to
    pub timescale: TimeScale,
    /// Sampling [Duration]
    pub sampling_period: Duration,
    /// Rawxm enable
    pub rawxm: bool,
    /// Ephemeris enable
    pub ephemeris: bool,
    /// ratio
    pub solutions_ratio: u16,

    /// Active [Constellation]s
    pub constellations: Vec<Constellation>,

    /// Serial number
    pub sn: Option<String>,

    /// RX-clock enabled
    pub rx_clock: bool,

    /// RX model
    pub model: Option<String>,

    /// Firmware version
    pub firmware: Option<String>,
}

impl Settings {
    pub fn to_ram_volatile_cfg(&self, buf: &mut Vec<u8>) {
        let mut cfg_data = Vec::<CfgVal>::new();

        if self.constellations.contains(&Constellation::GPS)
            || self.constellations.contains(&Constellation::QZSS)
        {
            cfg_data.push(CfgVal::SignalGpsEna(true));
            cfg_data.push(CfgVal::SignalQzssEna(true));
        } else {
            cfg_data.push(CfgVal::SignalGpsEna(false));
            cfg_data.push(CfgVal::SignalQzssEna(false));
        }

        if self.constellations.contains(&Constellation::Galileo) {
            cfg_data.push(CfgVal::SignalGalEna(true));
            cfg_data.push(CfgVal::SignalGalE1Ena(true));
            cfg_data.push(CfgVal::SignalGalE5bEna(true));
        } else {
            cfg_data.push(CfgVal::SignalGalEna(false));
            cfg_data.push(CfgVal::SignalGalE1Ena(false));
            cfg_data.push(CfgVal::SignalGalE5bEna(false));
        }

        if self.constellations.contains(&Constellation::QZSS) {
            cfg_data.push(CfgVal::SignalQzssL1caEna(true));
            cfg_data.push(CfgVal::SignalQzssL2cEna(true));
        } else {
            cfg_data.push(CfgVal::SignalQzssL1caEna(false));
            cfg_data.push(CfgVal::SignalQzssL2cEna(false));
        }

        if self.constellations.contains(&Constellation::Glonass) {
            cfg_data.push(CfgVal::SignalGloEna(true));
            cfg_data.push(CfgVal::SignalGloL1Ena(true));
            cfg_data.push(CfgVal::SignalGLoL2Ena(true));
        } else {
            cfg_data.push(CfgVal::SignalGloEna(false));
        }

        if self.constellations.contains(&Constellation::BeiDou) {
            cfg_data.push(CfgVal::SignalBdsEna(true));
        } else {
            cfg_data.push(CfgVal::SignalBdsEna(false));
        }

        if self.l1 {
            cfg_data.push(CfgVal::SignalGpsL1caEna(true));
            cfg_data.push(CfgVal::SignalBdsB1Ena(true));
            cfg_data.push(CfgVal::SignalBdsB2Ena(true));
            cfg_data.push(CfgVal::SignalGloL1Ena(true));
        } else {
            cfg_data.push(CfgVal::SignalGpsL1caEna(false));
            cfg_data.push(CfgVal::SignalBdsB1Ena(false));
            cfg_data.push(CfgVal::SignalBdsB2Ena(false));
            cfg_data.push(CfgVal::SignalGloL1Ena(false));
        }

        if self.l2 {
            cfg_data.push(CfgVal::SignalGpsL2cEna(true));
            cfg_data.push(CfgVal::SignalGLoL2Ena(true));
        } else {
            cfg_data.push(CfgVal::SignalGpsL2cEna(false));
            cfg_data.push(CfgVal::SignalGLoL2Ena(false));
        }

        if self.l5 {
            cfg_data.push(CfgVal::UndocumentedL5Enable(true));
        } else {
            cfg_data.push(CfgVal::UndocumentedL5Enable(false));
        }

        CfgValSetBuilder {
            version: 0,
            layers: CfgLayerSet::RAM,
            reserved1: 0,
            cfg_data: &cfg_data,
        }
        .extend_to(buf);
    }
}
