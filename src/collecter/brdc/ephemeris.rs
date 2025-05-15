use rinex::prelude::{Epoch, SV};

use ublox_lib::{
    RxmSfrbxGpsQzssFrame, RxmSfrbxGpsQzssFrame1, RxmSfrbxGpsQzssFrame2, RxmSfrbxGpsQzssFrame3,
    RxmSfrbxGpsQzssHow, RxmSfrbxGpsQzssSubframe, RxmSfrbxGpsQzssTelemetry, RxmSfrbxInterpreted,
};

use super::sfrbx::Sfrbx;

pub struct GpsQzssMessages {
    sv: SV,
    how: RxmSfrbxGpsQzssHow,
    tlm: RxmSfrbxGpsQzssTelemetry,
    frame1: Option<RxmSfrbxGpsQzssFrame1>,
    frame2: Option<RxmSfrbxGpsQzssFrame2>,
    frame3: Option<RxmSfrbxGpsQzssFrame3>,
}

impl GpsQzssMessages {
    pub fn is_complete(&self) -> bool {
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

    pub fn from_gps_qzss_frame(sv: SV, frame: RxmSfrbxGpsQzssFrame) -> Self {
        Self {
            sv,
            how: frame.how,
            tlm: frame.telemetry,
            frame1: match &frame.subframe {
                RxmSfrbxGpsQzssSubframe::Eph1(frame1) => Some(frame1.clone()),
                _ => None,
            },
            frame2: match &frame.subframe {
                RxmSfrbxGpsQzssSubframe::Eph2(frame2) => Some(frame2.clone()),
                _ => None,
            },
            frame3: match &frame.subframe {
                RxmSfrbxGpsQzssSubframe::Eph3(frame3) => Some(frame3.clone()),
                _ => None,
            },
        }
    }

    pub fn latch_gps_qzss_frame(&mut self, frame: RxmSfrbxGpsQzssFrame) {
        self.how = frame.how;
        self.tlm = frame.telemetry;

        match frame.subframe {
            RxmSfrbxGpsQzssSubframe::Eph1(frame1) => {
                self.frame1 = Some(frame1);
            },
            RxmSfrbxGpsQzssSubframe::Eph2(frame2) => {
                self.frame2 = Some(frame2);
            },
            RxmSfrbxGpsQzssSubframe::Eph3(frame3) => {
                self.frame3 = Some(frame3);
            },
        }
    }

    pub fn toc(&self, now: Epoch) -> Option<Epoch> {
        None
    }
}

pub enum EphMessages {
    GpsQzss(GpsQzssMessages),
}

impl EphMessages {
    pub fn from_sfrbx(sfrbx: Sfrbx) -> Self {
        match sfrbx.interpreted {
            RxmSfrbxInterpreted::GpsQzss(gps) => {
                Self::GpsQzss(GpsQzssMessages::from_gps_qzss_frame(sfrbx.sv, gps))
            },
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Self::GpsQzss(frame) => frame.is_complete(),
            _ => false,
        }
    }
}

pub struct EphBuffer {
    pub buffer: Vec<GpsQzssMessages>,
}

impl EphBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(8),
        }
    }

    pub fn latch_rxm_sfrbx(&mut self, sfrbx: Sfrbx) {
        match sfrbx.interpreted {
            RxmSfrbxInterpreted::GpsQzss(gps_qzss_frame) => {
                if let Some(data) = self.buffer.iter_mut().find(|k| k.sv == sfrbx.sv) {
                    data.latch_gps_qzss_frame(gps_qzss_frame);
                } else {
                    let msg = GpsQzssMessages::from_gps_qzss_frame(sfrbx.sv, gps_qzss_frame);
                    self.buffer.push(msg);
                }
            },
        }
    }

    pub fn has_pending_content(&self) -> bool {
        self.buffer.iter().filter(|k| k.is_complete()).count() > 0
    }
}
