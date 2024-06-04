#[cfg(feature = "benchmark")]
pub mod benchmark;

mod gov;
mod bank;
mod wasm;

pub use gov::{Gov, GovWithAppAccess};
pub use bank::Bank;
pub use test_tube::macros;
pub use wasm::Wasm;
pub use test_tube::module::Module;

pub fn type_url(url: &str) -> String {
    let mut t = "/".to_string();
    t.push_str(url);
    t
}