use crate::naming::NameType;
use serde::{Deserialize, Serialize};

pub type BenchResults = Vec<BenchResult>;

#[derive(Serialize, Deserialize, Clone)]
pub struct FinalizedGroup {
    pub group: String,
    pub name_type: NameType,
    pub results: BenchResults,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenchResult {
    pub name: String,
    pub gas: Gas,
    pub arch: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Gas {
    pub wanted: u128,
    pub used: u128,
}
