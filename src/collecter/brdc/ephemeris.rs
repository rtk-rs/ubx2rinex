use ublox::{MgaGloEphRef, MgaGpsEphRef};

use rinex::navigation::Ephemeris;

pub struct EphemerisBuilder {}

impl EphemerisBuilder {
    pub fn from_gps(gps: MgaGpsEphRef) -> Ephemeris {
        Ephemeris {
            clock_bias: gps.af0(),
            clock_drift: gps.af1(),
            clock_drift_rate: gps.af2(),
            orbits: Default::default(),
        }
    }

    pub fn from_glonass(glo: MgaGloEphRef) -> Ephemeris {
        Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits: Default::default(),
        }
    }
}
