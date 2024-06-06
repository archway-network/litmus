#[cfg(feature = "benchmark")]
pub mod benchmark;

mod gov;
mod bank;
mod wasm;
mod callback;
mod rewards;
mod cwica;
mod cwfees;

pub use gov::{Gov, GovWithAppAccess};
pub use bank::Bank;
pub use test_tube::macros;
pub use wasm::Wasm;
pub use callback::Callback;
pub use rewards::Rewards;
pub use cwica::CwIca;
pub use cwfees::CwFees;
pub use test_tube::module::Module;

pub fn type_url(url: &str) -> String {
    let mut t = "/".to_string();
    t.push_str(url);
    t
}