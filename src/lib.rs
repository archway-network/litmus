pub mod module;

use std::ffi::CString;
use std::str::FromStr;
pub use archway_proto;

#[cfg(feature = "benchmark")]
pub use module::benchmark::*;

use test_tube::cosmrs::proto::tendermint::v0_37::abci::{RequestDeliverTx, ResponseDeliverTx};
use cosmwasm_std::Coin;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use prost::Message;
use prost_types::Any;
pub use test_tube;
use test_tube::runner::error::DecodeError;
use test_tube::{Account, cosmrs, EncodeError, FeeSetting, redefine_as_go_string, Runner, RunnerError, RunnerExecuteResult, RunnerResult, SigningAccount};
use test_tube::bindings::{AccountNumber, AccountSequence, BeginBlock, EndBlock, Execute, GetBlockHeight, GetBlockTime, GetValidatorPrivateKey, IncreaseTime, InitAccount, InitTestEnv, Query, Simulate};
use test_tube::cosmrs::crypto::secp256k1::SigningKey;
use test_tube::cosmrs::{AccountId, tx};
use test_tube::cosmrs::tx::{Fee, SignerInfo};
use test_tube::runner::result::RawResult;

pub const FEE_DENOM: &str = "aarch";
// const ADDRESS_PREFIX: &str = "arch";
pub const ADDRESS_PREFIX: &str = "cosmos";
pub const CHAIN_ID: &str = "archway-1";

pub const DEFAULT_GAS_ADJUSTMENT: f64 = 1.4;
// pub const GAS_PRICE: u128 = 900_000_000_000;
const GAS_PRICE: u128 = 140_000_000_000;

pub fn aarch(amount: u128) -> Coin {
    Coin::new(amount, FEE_DENOM)
}

pub fn arch(amount: u128) -> Coin {
    aarch(amount * 10u128.pow(18))
}

pub struct ArchwayApp {
    id: u64,
    fee_denom: String,
    chain_id: String,
    address_prefix: String,
    default_gas_adjustment: f64,
}

impl ArchwayApp {
    pub fn new() -> Self {
        let id = unsafe { InitTestEnv() };
        Self {
            id,
            fee_denom: FEE_DENOM.to_string(),
            chain_id: CHAIN_ID.to_string(),
            address_prefix: ADDRESS_PREFIX.to_string(),
            default_gas_adjustment: DEFAULT_GAS_ADJUSTMENT,
        }
    }
}

impl ArchwayApp {
    pub fn get_block_time_nanos(&self) -> i64 {
        unsafe { GetBlockTime(self.id) }
    }
    
    /// Get the current block time in seconds
    pub fn get_block_time_seconds(&self) -> i64 {
        self.get_block_time_nanos() / 1_000_000_000i64
    }

    pub fn get_block_height(&self) -> i64 {
        unsafe { GetBlockHeight(self.id) }
    }

    pub fn increase_time(&self, seconds: u64) {
        unsafe {
            IncreaseTime(self.id, seconds.try_into().unwrap());
        }
    }

    pub fn get_first_validator_signing_account(&self) -> RunnerResult<SigningAccount> {
        let base64_priv = unsafe {
            let val_priv = GetValidatorPrivateKey(self.id, 0);
            CString::from_raw(val_priv)
        }
            .to_str()
            .map_err(DecodeError::Utf8Error)?
            .to_string();

        let secp256k1_priv = BASE64_STANDARD
            .decode(base64_priv)
            .map_err(DecodeError::Base64DecodeError)?;
        let signging_key = SigningKey::from_slice(&secp256k1_priv).map_err(|e| {
            let msg = e.to_string();
            DecodeError::SigningKeyDecodeError { msg }
        })?;

        Ok(SigningAccount::new(
            self.address_prefix.clone(),
            signging_key,
            FeeSetting::Auto {
                gas_price: Coin::new(GAS_PRICE, self.fee_denom.clone()),
                gas_adjustment: self.default_gas_adjustment,
            },
        ))
    }

    pub fn init_account(&self, coins: &[Coin]) -> RunnerResult<SigningAccount> {
        let mut coins = coins.to_vec();

        // invalid coins if denom are unsorted
        coins.sort_by(|a, b| a.denom.cmp(&b.denom));

        let coins_json = serde_json::to_string(&coins).map_err(EncodeError::JsonEncodeError)?;
        redefine_as_go_string!(coins_json);

        let base64_priv = unsafe {
            BeginBlock(self.id);
            let addr = InitAccount(self.id, coins_json);
            EndBlock(self.id);
            CString::from_raw(addr)
        }
            .to_str()
            .map_err(DecodeError::Utf8Error)?
            .to_string();

        let secp256k1_priv = BASE64_STANDARD
            .decode(base64_priv)
            .map_err(DecodeError::Base64DecodeError)?;
        let signging_key = SigningKey::from_slice(&secp256k1_priv).map_err(|e| {
            let msg = e.to_string();
            DecodeError::SigningKeyDecodeError { msg }
        })?;

        Ok(SigningAccount::new(
            self.address_prefix.clone(),
            signging_key,
            FeeSetting::Auto {
                gas_price: aarch(GAS_PRICE),
                gas_adjustment: self.default_gas_adjustment,
            },
        ))
    }

    pub fn get_account_sequence(&self, address: &str) -> u64 {
        redefine_as_go_string!(address);
        unsafe { AccountSequence(self.id, address) }
    }

    pub fn get_account_number(&self, address: &str) -> u64 {
        redefine_as_go_string!(address);
        unsafe { AccountNumber(self.id, address) }
    }

    pub fn init_accounts(&self, coins: &[Coin], count: u64) -> RunnerResult<Vec<SigningAccount>> {
        (0..count).map(|_| self.init_account(coins)).collect()
    }
    
    pub fn execute_multiple_with_granter<M, R>(
        &self,
        msgs: &[(M, &str)],
        signer: &SigningAccount,
        granter: Option<&str>
    ) -> RunnerExecuteResult<R>
        where
            M: ::prost::Message,
            R: ::prost::Message + Default,
    {
        let msgs = msgs
            .iter()
            .map(|(msg, type_url)| {
                let mut buf = Vec::new();
                M::encode(msg, &mut buf).map_err(EncodeError::ProtoEncodeError)?;

                Ok(Any {
                    type_url: type_url.to_string(),
                    value: buf,
                })
            })
            .collect::<Result<Vec<Any>, RunnerError>>()?;

        self.execute_multiple_raw_with_granter(msgs, signer, granter)
    }

    pub fn execute_multiple_raw_with_granter<R>(
        &self,
        msgs: Vec<Any>,
        signer: &SigningAccount,
        granter: Option<&str>
    ) -> RunnerExecuteResult<R>
        where
            R: ::prost::Message + Default,
    {
        unsafe {
            self.run_block(|| {
                // Set granter for the sim fee
                let mut sim_fee = self.default_simulation_fee();
                if let Some(granter) = granter {
                    sim_fee.granter = Some(AccountId::from_str(granter).unwrap())
                }
                
                let tx_sim_fee =
                    self.create_signed_tx(msgs.clone(), signer, sim_fee)?;
                let mut fee = self.calculate_fee(&tx_sim_fee, signer)?;
                
                if let Some(granter) = granter {
                    fee.granter = Some(AccountId::from_str(granter).unwrap())
                }

                let tx = self.create_signed_tx(msgs.clone(), signer, fee)?.into();

                let mut buf = Vec::new();
                RequestDeliverTx::encode(&RequestDeliverTx { tx }, &mut buf)
                    .map_err(EncodeError::ProtoEncodeError)?;

                let base64_req = BASE64_STANDARD.encode(buf);
                redefine_as_go_string!(base64_req);

                let res = Execute(self.id, base64_req);
                let res = RawResult::from_non_null_ptr(res).into_result()?;

                ResponseDeliverTx::decode(res.as_slice())
                    .map_err(DecodeError::ProtoDecodeError)?
                    .try_into()
            })
        }
    }

    pub fn default_simulation_fee(&self) -> Fee {
        Fee::from_amount_and_gas(
            cosmrs::Coin {
                denom: self.fee_denom.parse().unwrap(),
                amount: GAS_PRICE,
            },
            0u64,
        )
    }

    pub fn simulate_tx_bytes(
        &self,
        tx_bytes: &[u8],
    ) -> RunnerResult<cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo> {
        let base64_tx_bytes = BASE64_STANDARD.encode(tx_bytes);
        redefine_as_go_string!(base64_tx_bytes);

        unsafe {
            let res = Simulate(self.id, base64_tx_bytes);
            let res = RawResult::from_non_null_ptr(res).into_result()?;

            cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo::decode(res.as_slice())
                .map_err(DecodeError::ProtoDecodeError)
                .map_err(RunnerError::DecodeError)
        }
    }
    
    pub fn calculate_fee(&self, tx_bytes: &[u8], fee_payer: &SigningAccount) -> RunnerResult<Fee> {
        match &fee_payer.fee_setting() {
            FeeSetting::Auto {
                gas_price,
                gas_adjustment,
            } => {
                let gas_info = self.simulate_tx_bytes(tx_bytes)?;
                let gas_limit = ((gas_info.gas_used as f64) * (gas_adjustment)).ceil() as u64;
                let amount = cosmrs::Coin {
                    denom: self.fee_denom.parse().unwrap(),
                    amount: (((gas_limit as f64) * (gas_price.amount.u128() as f64)).ceil() as u64)
                        .into(),
                };

                Ok(Fee::from_amount_and_gas(amount, gas_limit))
            }
            FeeSetting::Custom { amount, gas_limit } => Ok(Fee::from_amount_and_gas(
                cosmrs::Coin {
                    denom: amount.denom.parse().unwrap(),
                    amount: amount.amount.to_string().parse().unwrap(),
                },
                *gas_limit,
            )),
        }
    }

    fn create_signed_tx<I>(
        &self,
        msgs: I,
        signer: &SigningAccount,
        fee: Fee,
    ) -> RunnerResult<Vec<u8>>
        where
            I: IntoIterator<Item = cosmrs::Any>,
    {
        let tx_body = tx::Body::new(msgs, "", 0u32);
        let addr = signer.address();

        let seq = self.get_account_sequence(&addr);
        let account_number = self.get_account_number(&addr);

        let signer_info = SignerInfo::single_direct(Some(signer.public_key()), seq);

        let chain_id = self
            .chain_id
            .parse()
            .expect("parse const str of chain id should never fail");

        let auth_info = signer_info.auth_info(fee);
        let sign_doc = tx::SignDoc::new(&tx_body, &auth_info, &chain_id, account_number)
            .map_err(EncodeError::from_proto_error_report)?;

        let tx_raw = sign_doc
            .sign(signer.signing_key())
            .map_err(EncodeError::from_proto_error_report)?;

        tx_raw
            .to_bytes()
            .map_err(EncodeError::from_proto_error_report)
            .map_err(RunnerError::EncodeError)
    }

    unsafe fn run_block<T, E>(&self, execution: impl Fn() -> Result<T, E>) -> Result<T, E> {
        unsafe { BeginBlock(self.id) };
        match execution() {
            ok @ Ok(_) => {
                unsafe { EndBlock(self.id) };
                ok
            }
            err @ Err(_) => {
                unsafe { EndBlock(self.id) };
                err
            }
        }
    }
    
}

impl Default for ArchwayApp {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Runner<'a> for ArchwayApp {
    fn execute_multiple<M, R>(
        &self,
        msgs: &[(M, &str)],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        M: ::prost::Message,
        R: ::prost::Message + Default,
    {
        self.execute_multiple_with_granter(msgs, signer, None)
    }

    fn execute_multiple_raw<R>(
        &self,
        msgs: Vec<Any>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        R: prost::Message + Default,
    {
        self.execute_multiple_raw_with_granter(msgs, signer, None)
    }

    fn execute_tx(&self, tx_bytes: &[u8]) -> RunnerResult<ResponseDeliverTx> {
        unsafe {
            self.run_block(|| {
                let request_devlier_tx = RequestDeliverTx {
                    tx: tx_bytes.to_vec().into(),
                };

                let base64_req = BASE64_STANDARD.encode(request_devlier_tx.encode_to_vec());
                redefine_as_go_string!(base64_req);

                let res = Execute(self.id, base64_req);
                let res = RawResult::from_non_null_ptr(res).into_result()?;

                ResponseDeliverTx::decode(res.as_slice())
                    .map_err(DecodeError::ProtoDecodeError)
                    .map_err(Into::into)
            })
        }
    }

    fn query<Q, R>(&self, path: &str, q: &Q) -> RunnerResult<R>
        where
            Q: ::prost::Message,
            R: ::prost::Message + Default,
    {
        let mut buf = Vec::new();

        Q::encode(q, &mut buf).map_err(EncodeError::ProtoEncodeError)?;

        let base64_query_msg_bytes = BASE64_STANDARD.encode(buf);
        redefine_as_go_string!(path);
        redefine_as_go_string!(base64_query_msg_bytes);

        unsafe {
            let res = Query(self.id, path, base64_query_msg_bytes);
            let res = RawResult::from_non_null_ptr(res).into_result()?;
            R::decode(res.as_slice())
                .map_err(DecodeError::ProtoDecodeError)
                .map_err(RunnerError::DecodeError)
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_schema::cw_serde;
    use std::option::Option::None;

    use cosmwasm_std::{coins, Coin, Uint128};
    use cw1_whitelist::msg::{ExecuteMsg, InstantiateMsg};
    use archway_proto::cosmos::bank::v1beta1::QueryAllBalancesRequest;
    use serde::Serialize;

    use crate::{aarch, arch, ArchwayApp};
    use test_tube::account::{Account, FeeSetting};
    use test_tube::module::Module;
    use test_tube::runner::*;
    use crate::module::{Bank, Wasm};

    pub mod netwars_msgs {
        use cosmwasm_std::{Addr, Uint128};
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        pub struct InstantiateMsg {
            pub archid_registry: Option<Addr>,
            pub expiration: u64,
            pub min_deposit: Uint128,
            pub extensions: u64,
            pub stale: u64,
            pub reset_length: u64,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        #[serde(rename_all = "snake_case")]
        pub enum ExecuteMsg {
            Deposit {},
        }
    }

    #[test]
    fn netwars() {
        let app = ArchwayApp::default();
        let accounts = app.init_accounts(&vec![arch(100)], 2).unwrap();
        let admin = accounts.get(0).unwrap();
        let depositor = accounts.get(0).unwrap();

        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/network_wars.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &admin)
            .unwrap()
            .data
            .code_id;

        let contract_addr = wasm
            .instantiate(
                code_id,
                &netwars_msgs::InstantiateMsg {
                    archid_registry: None,
                    expiration: 604800,
                    min_deposit: Uint128::from(1000000000000000000_u128),
                    extensions: 3600,
                    stale: 604800,
                    reset_length: 604800,
                },
                Some(&admin.address()),
                Some("netwars"),
                &[],
                &admin,
            )
            .unwrap()
            .data
            .address;

        let res = wasm
            .execute(
                &contract_addr,
                &netwars_msgs::ExecuteMsg::Deposit {},
                &[arch(1)],
                &depositor,
            )
            .unwrap();
        println!("   Chain | Gas Wanted | Gas Used");
        println!(
            "TestTube |   {}   | {}",
            res.gas_info.gas_wanted, res.gas_info.gas_used
        );
    }

    #[test]
    fn marketplace_test() {
        let app = ArchwayApp::default();
        let admin = app.init_account(&vec![arch(100)]).unwrap();

        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/low_gas_demo.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &admin)
            .unwrap()
            .data
            .code_id;

        #[derive(Serialize)]
        struct InstMsg {}

        let contract_addr = wasm
            .instantiate(
                code_id,
                &InstMsg {},
                Some(&admin.address()),
                Some("test_contract"),
                &[],
                &admin,
            )
            .unwrap()
            .data
            .address;

        #[cw_serde]
        pub enum ExecMsg {
            Iterate { iterations: u64 },
            EmptyLoad {},
        }

        let res = wasm
            .execute::<ExecMsg>(&contract_addr, &ExecMsg::EmptyLoad {}, &[], &admin)
            .unwrap();
        println!("   Chain | Gas Wanted | Gas Used");
        println!("TestNet  |   187574   | 185786");
        println!(
            "TestTube |   {}   | {}",
            res.gas_info.gas_wanted, res.gas_info.gas_used
        );
    }

    #[test]
    fn test_init_accounts() {
        let app = ArchwayApp::default();
        let accounts = app
            .init_accounts(&coins(100_000_000_000, "aarch"), 3)
            .unwrap();

        assert!(accounts.get(0).is_some());
        assert!(accounts.get(1).is_some());
        assert!(accounts.get(2).is_some());
        assert!(accounts.get(3).is_none());
    }

    #[test]
    fn test_get_and_set_block_timestamp() {
        let app = ArchwayApp::default();

        let block_time_nanos = app.get_block_time_nanos();
        let block_time_seconds = app.get_block_time_seconds();

        app.increase_time(10u64);

        assert_eq!(
            app.get_block_time_nanos(),
            block_time_nanos + 10_000_000_000
        );
        assert_eq!(app.get_block_time_seconds(), block_time_seconds + 10);
    }

    #[test]
    fn test_get_block_height() {
        let app = ArchwayApp::default();

        // Governance transactions fix
        assert_eq!(app.get_block_height(), 1i64);

        app.increase_time(10u64);

        assert_eq!(app.get_block_height(), 2i64);
    }

    #[test]
    fn test_wasm_execute_and_query() {
        use cw1_whitelist::msg::*;

        let app = ArchwayApp::default();
        let accs = app
            .init_accounts(
                &[
                    Coin::new(1_000_000_000_000, "uatom"),
                    Coin::new(1_000_000_000_000_000_000_000_000, "aarch"),
                ],
                2,
            )
            .unwrap();
        let admin = &accs[0];
        let new_admin = &accs[1];

        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, admin)
            .unwrap()
            .data
            .code_id;
        assert_eq!(code_id, 1);

        // initialize admins and check if the state is correct
        let init_admins = vec![admin.address()];
        let contract_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    admins: init_admins.clone(),
                    mutable: true,
                },
                Some(&admin.address()),
                Some("cw1_whitelist"),
                &[],
                admin,
            )
            .unwrap()
            .data
            .address;
        let admin_list = wasm
            .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
            .unwrap();
        assert_eq!(admin_list.admins, init_admins);
        assert!(admin_list.mutable);

        // update admin and check again
        let new_admins = vec![new_admin.address()];
        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::UpdateAdmins {
                admins: new_admins.clone(),
            },
            &[],
            admin,
        )
        .unwrap();

        let admin_list = wasm
            .query::<QueryMsg, AdminListResponse>(&contract_addr, &QueryMsg::AdminList {})
            .unwrap();

        assert_eq!(admin_list.admins, new_admins);
        assert!(admin_list.mutable);
    }

    #[test]
    fn test_custom_fee() {
        let app = ArchwayApp::default();
        let initial_balance = 1_000_000_000_000_000_000_000;
        let alice = app
            .init_account(&coins(initial_balance, "aarch"))
            .unwrap();
        let bob = app
            .init_account(&coins(initial_balance, "aarch"))
            .unwrap();

        let amount = Coin::new(1_000_000, "aarch");
        let gas_limit = 100_000_000;

        // use FeeSetting::Auto by default, so should not equal newly custom fee setting
        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/cw1_whitelist.wasm").unwrap();
        let res = wasm.store_code(&wasm_byte_code, None, &alice).unwrap();

        assert_ne!(res.gas_info.gas_wanted, gas_limit);

        //update fee setting
        let bob = bob.with_fee_setting(FeeSetting::Custom {
            amount: amount.clone(),
            gas_limit,
        });
        let res = wasm.store_code(&wasm_byte_code, None, &bob).unwrap();

        let bob_balance = Bank::new(&app)
            .query_all_balances(&QueryAllBalancesRequest {
                address: bob.address(),
                pagination: None,
            })
            .unwrap()
            .balances
            .into_iter()
            .find(|c| c.denom == "aarch")
            .unwrap()
            .amount
            .parse::<u128>()
            .unwrap();

        assert_eq!(res.gas_info.gas_wanted, gas_limit);
        assert_eq!(bob_balance, initial_balance - amount.amount.u128());
    }
}
