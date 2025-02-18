use crate::module::type_url;
use archway_proto::cosmwasm::wasm::v1::{
    AccessConfig, MsgExecuteContract, MsgExecuteContractResponse, MsgInstantiateContract,
    MsgInstantiateContractResponse, MsgMigrateContract, MsgMigrateContractResponse, MsgStoreCode,
    MsgStoreCodeResponse, QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};
use cosmwasm_std::Coin;
use prost::Name;
use serde::{de::DeserializeOwned, Serialize};
use test_tube::{
    Account, DecodeError, EncodeError, Runner, RunnerError, RunnerExecuteResult, RunnerResult,
    SigningAccount,
};

pub struct Wasm<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> super::Module<'a, R> for Wasm<'a, R> {
    fn new(runner: &'a R) -> Self {
        Wasm { runner }
    }
}

impl<'a, R> Wasm<'a, R>
where
    R: Runner<'a>,
{
    pub fn store_code(
        &self,
        wasm_byte_code: &[u8],
        instantiate_permission: Option<AccessConfig>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgStoreCodeResponse> {
        self.runner.execute(
            MsgStoreCode {
                sender: signer.address(),
                wasm_byte_code: wasm_byte_code.to_vec(),
                instantiate_permission,
            },
            &type_url(&MsgStoreCode::full_name()),
            signer,
        )
    }

    pub fn migrate<M>(
        &self,
        contract: String,
        code_id: u64,
        msg: &M,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgMigrateContractResponse>
    where
        M: ?Sized + Serialize,
    {
        self.runner.execute(
            MsgMigrateContract {
                sender: signer.address(),
                contract,
                code_id,
                msg: serde_json::to_vec(msg).map_err(EncodeError::JsonEncodeError)?,
            },
            &type_url(&MsgMigrateContract::full_name()),
            signer,
        )
    }

    pub fn instantiate<M>(
        &self,
        code_id: u64,
        msg: &M,
        admin: Option<&str>,
        label: Option<&str>,
        funds: &[Coin],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgInstantiateContractResponse>
    where
        M: ?Sized + Serialize,
    {
        self.runner.execute(
            MsgInstantiateContract {
                sender: signer.address(),
                admin: admin.unwrap_or_default().to_string(),
                code_id,
                label: label.unwrap_or(" ").to_string(), // empty string causes panic
                msg: serde_json::to_vec(msg).map_err(EncodeError::JsonEncodeError)?,
                funds: funds
                    .iter()
                    .map(|c| archway_proto::cosmos::base::v1beta1::Coin {
                        denom: c.denom.parse().unwrap(),
                        amount: format!("{}", c.amount.u128()),
                    })
                    .collect(),
            },
            &type_url(&MsgInstantiateContract::full_name()),
            signer,
        )
    }

    pub fn execute<M>(
        &self,
        contract: &str,
        msg: &M,
        funds: &[Coin],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse>
    where
        M: ?Sized + Serialize,
    {
        self.runner.execute(
            MsgExecuteContract {
                sender: signer.address(),
                msg: serde_json::to_vec(msg).map_err(EncodeError::JsonEncodeError)?,
                funds: funds
                    .iter()
                    .map(|c| archway_proto::cosmos::base::v1beta1::Coin {
                        denom: c.denom.parse().unwrap(),
                        amount: format!("{}", c.amount.u128()),
                    })
                    .collect(),
                contract: contract.to_owned(),
            },
            &type_url(&MsgExecuteContract::full_name()),
            signer,
        )
    }

    pub fn query<M, Res>(&self, contract: &str, msg: &M) -> RunnerResult<Res>
    where
        M: ?Sized + Serialize,
        Res: DeserializeOwned,
    {
        let res = self
            .runner
            .query::<QuerySmartContractStateRequest, QuerySmartContractStateResponse>(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                &QuerySmartContractStateRequest {
                    address: contract.to_owned(),
                    query_data: serde_json::to_vec(msg).map_err(EncodeError::JsonEncodeError)?,
                },
            )?;

        serde_json::from_slice(&res.data)
            .map_err(DecodeError::JsonDecodeError)
            .map_err(RunnerError::DecodeError)
    }
}
