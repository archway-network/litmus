mod authz;
mod bank;
mod callback;
mod cwerrors;
mod cwfees;
mod cwica;
mod distribution;
mod gov;
mod rewards;
mod staking;
mod wasm;

pub use authz::Authz;
pub use bank::Bank;
pub use callback::Callback;
pub use cwerrors::CwErrors;
pub use cwfees::CwFees;
pub use cwica::CwIca;
pub use distribution::Distribution;
pub use gov::{Gov, GovWithAppAccess};
pub use rewards::Rewards;
pub use staking::Staking;
pub use test_tube::macros;
pub use test_tube::module::Module;
pub use wasm::Wasm;

pub fn type_url(url: &str) -> String {
    let mut t = "/".to_string();
    t.push_str(url);
    t
}
