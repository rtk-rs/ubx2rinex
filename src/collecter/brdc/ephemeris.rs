use rinex::{
    navigation::Ephemeris,
    prelude::{Constellation, Epoch, SV},
};

use ublox_lib::{
    GpsEphFrame1, GpsEphFrame2, GpsEphFrame3, GpsFrame, GpsHowWord, GpsSubframe, GpsTelemetryWord,
    RxmSfrbxInterpreted,
};

use super::sfrbx::Sfrbx;

pub struct GpsEphMessages {
    sv: SV,
    how: GpsHowWord,
    tlm: GpsTelemetryWord,
    frame1: Option<GpsEphFrame1>,
    frame2: Option<GpsEphFrame2>,
    frame3: Option<GpsEphFrame3>,
}

impl GpsEphMessages {
    pub fn is_ready(&self) -> bool {
        if let Some(frame1) = self.frame1.as_ref() {
            if let Some(frame2) = self.frame2.as_ref() {
                if let Some(frame3) = self.frame3.as_ref() {
                    if frame2.iode == frame3.iode {
                        if frame1.iodc as u8 == frame2.iode {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn from_gps_frame(sv: SV, gps: GpsFrame) -> Self {
        Self {
            sv,
            how: gps.how,
            tlm: gps.telemetry,
            frame1: match &gps.subframe {
                GpsSubframe::Eph1(frame1) => Some(frame1.clone()),
                _ => None,
            },
            frame2: match &gps.subframe {
                GpsSubframe::Eph2(frame2) => Some(frame2.clone()),
                _ => None,
            },
            frame3: match &gps.subframe {
                GpsSubframe::Eph3(frame3) => Some(frame3.clone()),
                _ => None,
            },
        }
    }

    pub fn latch_gps_frame(&mut self, gps: GpsFrame) {
        self.how = gps.how;
        self.tlm = gps.telemetry;

        match gps.subframe {
            GpsSubframe::Eph1(frame1) => {
                self.frame1 = Some(frame1);
            },
            GpsSubframe::Eph2(frame2) => {
                self.frame2 = Some(frame2);
            },
            GpsSubframe::Eph3(frame3) => {
                self.frame3 = Some(frame3);
            },
        }
    }

    pub fn toc(&self, now: Epoch) -> Option<Epoch> {
        None
    }
}

pub enum EphMessages {
    GPS(GpsEphMessages),
}

impl EphMessages {
    pub fn from_sfrbx(sfrbx: Sfrbx) -> Self {
        match sfrbx.interpreted {
            RxmSfrbxInterpreted::GPS(gps) => {
                Self::GPS(GpsEphMessages::from_gps_frame(sfrbx.sv, gps))
            },
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            Self::GPS(gps) => gps.is_ready(),
            _ => false,
        }
    }
}

pub struct EphBuffer {
    pub buffer: Vec<GpsEphMessages>,
}

impl EphBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(8),
        }
    }

    pub fn latch_rxm_sfrbx(&mut self, sfrbx: Sfrbx) {
        match sfrbx.interpreted {
            RxmSfrbxInterpreted::GPS(gps_frame) => {
                if let Some(data) = self.buffer.iter_mut().find(|k| k.sv == sfrbx.sv) {
                    data.latch_gps_frame(gps_frame);
                } else {
                    let msg = GpsEphMessages::from_gps_frame(sfrbx.sv, gps_frame);
                    self.buffer.push(msg);
                }
            },
        }
    }

    pub fn has_pending_content(&self) -> bool {
        self.buffer.iter().filter(|k| k.is_ready()).count() > 0
    }
}
