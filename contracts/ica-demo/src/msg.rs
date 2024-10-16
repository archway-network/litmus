use archway_proto::archway::cwerrors::v1::SudoError;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateICA {
        connection_id: String,
    },
    ExecuteICA {
        connection_id: String,
        grantee: String,
        delegator: String,
        validator: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SudoMsg {
    Ica {
        account_registered: Option<AccountRegistered>,
        tx_executed: Option<ICAResponse>,
    },
    Error(SudoError),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    IcaAccount { connection_id: String },
}

#[cw_serde]
pub struct IcaMsg {
    pub account_registered: Option<AccountRegistered>,
    pub tx_executed: Option<ICAResponse>,
}

#[cw_serde]
pub struct AccountRegistered {
    pub counterparty_address: String,
}

#[cw_serde]
pub struct ICAResponse {
    pub packet: RequestPacket,
    pub data: Binary,
}

#[cw_serde]
pub struct RequestPacket {
    pub sequence: Option<u64>,
    pub source_port: Option<String>,
    pub source_channel: Option<String>,
    pub destination_port: Option<String>,
    pub destination_channel: Option<String>,
    pub data: Option<Binary>,
    pub timeout_height: Option<RequestPacketTimeoutHeight>,
    pub timeout_timestamp: Option<u64>,
}

#[cw_serde]
pub struct RequestPacketTimeoutHeight {
    pub revision_number: Option<u64>,
    pub revision_height: Option<u64>,
}
