use rinex::prelude::{Carrier, Epoch, SV};

#[derive(Debug, Clone, Copy)]
pub struct Rawxm {
    pub t: Epoch,
    pub sv: SV,
    pub pr: f64,
    pub cp: f64,
    pub dop: f32,
    pub cno: u8,
    pub carrier: Carrier,
}

impl std::fmt::Display for Rawxm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}) {} pr={:.7E} cp={:.7E} dop={:.7E} cno={}",
            self.t, self.sv, self.carrier, self.pr, self.cp, self.dop, self.cno,
        )
    }
}
