use archway_proto::archway::cwica::v1::IcaSuccess;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IcaAccount {
    /// Ica address in the other chain
    pub ica_host_address: String,
    /// User's delegator address in the other chain
    pub delegator_address: String,
}

pub const PENDING_ACCOUNT: Item<(String, String)> = Item::new("pending");
pub const ICA_ACCOUNTS: Map<String, IcaAccount> = Map::new("ica_accounts");
pub const ICA_HISTORY: Item<Vec<IcaSuccess>> = Item::new("ica_history");
