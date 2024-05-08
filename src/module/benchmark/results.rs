use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;

// We want sorted data in all of our generated files
pub type GroupResults = BTreeMap<String, BenchResult>;

/// Results of a bench
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct BenchResult {
    pub gas_wanted: u64,
    pub gas_used: u64,
}
