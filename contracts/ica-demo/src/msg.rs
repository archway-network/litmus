use archway_proto::archway::cwerrors::v1::SudoError;
use archway_proto::archway::cwica::v1::IcaSuccess;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateICA {
        /// Chain name
        chain_name: String,
        /// User's address in the other chain
        delegator_address: String,
        /// Relayer connection ID, ideally this gets offloaded to another smart contract
        connection_id: String,
    },
    InitDelegation {
        /// Relayer connection ID, this gets offloaded in the future
        connection_id: String,
        chain_name: String,
        /// Validator address
        validator: String,
        /// Staking amount
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    Ica(IcaSuccess),
    Error(SudoError),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IcaAccount { connection_id: String },
}
