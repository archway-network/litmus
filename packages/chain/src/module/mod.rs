mod bank;
mod callback;
mod cwfees;
mod cwica;
mod gov;
mod rewards;
mod wasm;

pub use bank::Bank;
pub use callback::Callback;
pub use cwfees::CwFees;
pub use cwica::CwIca;
pub use gov::{Gov, GovWithAppAccess};
pub use rewards::Rewards;
pub use test_tube::macros;
pub use test_tube::module::Module;
pub use wasm::Wasm;

pub fn type_url(url: &str) -> String {
    let mut t = "/".to_string();
    t.push_str(url);
    t
}
