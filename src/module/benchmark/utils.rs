use crate::ArchwayApp;
use cosmwasm_std::Coin;
use test_tube::SigningAccount;

/// Setup function must export these params for execution to work
pub struct Setup<M> {
    pub app: ArchwayApp,
    // Contract address to execute
    pub contract: String,
    pub signer: SigningAccount,
    pub funds: Vec<Coin>,
    pub msg: M,
}

impl<M> Setup<M> {
    pub fn new(
        app: ArchwayApp,
        contract: String,
        signer: SigningAccount,
        funds: Vec<Coin>,
        msg: M,
    ) -> Self {
        Self {
            app,
            contract,
            signer,
            funds,
            msg,
        }
    }
}

pub struct BenchConfig {
    // How many iterations to perform
    pub path: String,
    pub name: String,
    pub history: Vec<BenchSaveConfig>,
    /// When false, it will display every single bench available, may cause issues
    pub truncate_benches: bool,
}

impl BenchConfig {
    pub fn get_path(&self) -> String {
        format!("{}/{}", self.path, self.name)
    }
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            // Samples do not vary per execution
            path: "./".to_string(),
            name: "test_tube_bench".to_string(),
            history: vec![BenchSaveConfig::save_last()],
            truncate_benches: true,
        }
    }
}

// TODO: add some sort of verification
//  new_results_name cannot be config
//  file limit cannot be 0
//  file rotation cannot be size 0 or 1

pub struct BenchSaveConfig {
    /// Config name, and where everything will be stored
    pub name: String,
    /// Store the files manifest, and delete the last n results
    pub file_limit: Option<usize>,
    /// Config name for new results
    pub new_results_name: String,
    /// Attempt to rotate all files with the given names in order
    pub file_rotation: Option<Vec<String>>,
}

impl BenchSaveConfig {
    /// Save results as results.json, and store no file history
    pub fn no_history() -> Self {
        Self {
            name: "current".to_string(),
            file_limit: None,
            file_rotation: None,
            new_results_name: "results".to_string(),
        }
    }

    /// Saves the current result as new.json and the previous as base.json
    pub fn save_last() -> Self {
        Self {
            name: "base_last".to_string(),
            file_limit: None,
            new_results_name: "new".to_string(),
            file_rotation: Some(vec!["base".to_string(), "new".to_string()]),
        }
    }

    // old, base, new

    pub fn package_version(limit: usize) -> Self {
        Self {
            name: "package_version".to_string(),
            file_limit: Some(limit),
            new_results_name: format!(
                "v{}_{}_{}",
                env!("CARGO_PKG_VERSION_MAJOR"),
                env!("CARGO_PKG_VERSION_MINOR"),
                env!("CARGO_PKG_VERSION_PATCH")
            ),
            file_rotation: None,
        }
    }
}
