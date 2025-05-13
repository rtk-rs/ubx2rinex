use rinex::prelude::SV;
use ublox_lib::RxmSfrbxInterpreted;

#[derive(Clone, Debug)]
pub struct Sfrbx {
    pub sv: SV,
    pub interpreted: RxmSfrbxInterpreted,
}
