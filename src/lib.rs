pub mod module;

pub use archway_proto;

#[cfg(feature = "benchmark")]
pub use module::benchmark::*;

use crate::module::GovWithAppAccess;
use test_tube::cosmrs::proto::tendermint::v0_37::abci::ResponseDeliverTx;
use cosmwasm_std::{Coin, CosmosMsg};
use archway_proto::cosmos::params::v1beta1::{ParamChange, ParameterChangeProposal};
use prost::{Message, Name};
use prost_types::Any;
use serde::de::DeserializeOwned;
pub use test_tube;
use test_tube::{
    Account, BaseApp, FeeSetting, Runner, RunnerExecuteResult, RunnerResult, SigningAccount,
};

const FEE_DENOM: &str = "aarch";
// const ADDRESS_PREFIX: &str = "arch";
const ADDRESS_PREFIX: &str = "cosmos";
const CHAIN_ID: &str = "archway-1";

const DEFAULT_GAS_ADJUSTMENT: f64 = 1.4;
const GAS_PRICE: u128 = 900_000_000_000;
// const GAS_PRICE: u128 = 140_000_000_000;

pub fn aarch(amount: u128) -> Coin {
    Coin::new(amount, FEE_DENOM)
}

pub fn arch(amount: u128) -> Coin {
    aarch(amount * 10u128.pow(18))
}

pub struct ArchwayApp {
    inner: BaseApp,
}

impl ArchwayApp {
    pub fn new() -> Self {
        let app = ArchwayApp {
            inner: BaseApp::new(FEE_DENOM, CHAIN_ID, ADDRESS_PREFIX, DEFAULT_GAS_ADJUSTMENT),
        };

        let proposer = app.init_account(&[arch(100)]).unwrap();

        let gov = GovWithAppAccess::new(&app);
        gov.propose_and_execute(
            ParameterChangeProposal::full_name(),
            ParameterChangeProposal {
                title: "Change gas price to current nets".to_string(),
                description: "A perfectly descriptive description".to_string(),
                changes: vec![ParamChange {
                    subspace: "rewards".to_string(),
                    key: "MinPriceOfGas".to_string(),
                    value:
                        "{ \"denom\": \"aarch\", \"amount\": \"140000000000.000000000000000000\"}"
                            .to_string(),
                }],
            },
            proposer.address(),
            &proposer,
        )
        .unwrap();

        app
    }
}

impl ArchwayApp {
    /// Get the current block time in seconds
    pub fn get_block_time_seconds(&self) -> i64 {
        self.inner.get_block_time_nanos() / 1_000_000_000i64
    }

    /// Inits accounts with default fee settings
    pub fn init_account(&self, coins: &[Coin]) -> RunnerResult<SigningAccount> {
        self.inner.init_account(coins).map(|acc| {
            acc.with_fee_setting(FeeSetting::Auto {
                gas_price: aarch(GAS_PRICE),
                gas_adjustment: DEFAULT_GAS_ADJUSTMENT,
            })
        })
    }

    pub fn init_accounts(&self, coins: &[Coin], count: u64) -> RunnerResult<Vec<SigningAccount>> {
        (0..count).map(|_| self.init_account(coins)).collect()
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
        self.inner.execute_multiple(msgs, signer)
    }

    fn execute_multiple_raw<R>(
        &self,
        msgs: Vec<Any>,
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<R>
    where
        R: prost::Message + Default,
    {
        self.inner.execute_multiple_raw(msgs, signer)
    }

    fn execute_tx(&self, tx_bytes: &[u8]) -> RunnerResult<ResponseDeliverTx> {
        self.inner.execute_tx(tx_bytes)
    }

    fn execute_cosmos_msgs<S>(
        &self,
        msgs: &[CosmosMsg],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<S>
    where
        S: Message + Default,
    {
        self.inner.execute_cosmos_msgs(msgs, signer)
    }

    fn query<Q, R>(&self, path: &str, q: &Q) -> RunnerResult<R>
    where
        Q: ::prost::Message,
        R: ::prost::Message + DeserializeOwned + Default,
    {
        self.inner.query(path, q)
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
            .inner
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

        let block_time_nanos = app.inner.get_block_time_nanos();
        let block_time_seconds = app.get_block_time_seconds();

        app.inner.increase_time(10u64);

        assert_eq!(
            app.inner.get_block_time_nanos(),
            block_time_nanos + 10_000_000_000
        );
        assert_eq!(app.get_block_time_seconds(), block_time_seconds + 10);
    }

    #[test]
    fn test_get_block_height() {
        let app = ArchwayApp::default();

        // Governance transactions fix
        assert_eq!(app.inner.get_block_height(), 5i64);

        app.inner.increase_time(10u64);

        assert_eq!(app.inner.get_block_height(), 6i64);
    }

    #[test]
    fn test_wasm_execute_and_query() {
        use cw1_whitelist::msg::*;

        let app = ArchwayApp::default();
        let accs = app
            .inner
            .init_accounts(
                &[
                    Coin::new(1_000_000_000_000, "uatom"),
                    Coin::new(1_000_000_000_000, "aarch"),
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
        let initial_balance = 1_000_000_000_000;
        let alice = app
            .inner
            .init_account(&coins(initial_balance, "aarch"))
            .unwrap();
        let bob = app
            .inner
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
