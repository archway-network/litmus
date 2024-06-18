use std::sync::Arc;
use crate::naming::Naming;
use crate::results::{BenchResult, BenchResults, Gas};
use archway_test_tube::archway_proto::cosmos::bank::v1beta1::QueryBalanceRequest;
use archway_test_tube::module::{Bank, Module, Wasm};
use archway_test_tube::test_tube::{Account, SigningAccount};
use archway_test_tube::{ArchwayApp, FEE_DENOM};
use cosmwasm_std::Coin;
use serde::Serialize;

/// Output msg for the job benching
pub struct Setup<MSG> {
    pub contract: String,
    pub signer: Arc<SigningAccount>,
    pub funds: Vec<Coin>,
    pub msg: MSG,
}

pub struct JobResult {
    pub group_id: usize,
    pub results: BenchResults,
}

pub trait Job: Send + Sync {
    /// Sets the job group ID to notify when its finished
    fn set_group_id(&mut self, id: usize);
    fn get_group_id(&self) -> usize;
    fn run(&self, app: ArchwayApp) -> JobResult;
}

// A single threaded group, app state is saved across benches
pub struct Continuous<STATE, PARAM, MSG>
where PARAM: Send + Sync{
    pub id: usize,
    pub parameters: Vec<PARAM>,
    pub setup: Box<dyn Fn(&ArchwayApp) -> STATE + Send + Sync>,
    pub update: Box<dyn Fn(&ArchwayApp, &mut STATE, &PARAM) -> Setup<MSG> + Send + Sync>,
}

impl<STATE, PARAM: Naming + Send + Sync, MSG: Sized + Serialize + Send + Sync> Job for Continuous<STATE, PARAM, MSG> {
    fn set_group_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_group_id(&self) -> usize {
        self.id
    }

    fn run(&self, app: ArchwayApp) -> JobResult {
        let mut state = (self.setup)(&app);

        let mut results = vec![];
        for param in self.parameters.iter() {
            let bench_name = param.name();
            let tx_params = (self.update)(&app, &mut state, param);
            results.push(bench_msg(&app, bench_name, tx_params));
        }

        JobResult {
            group_id: self.id,
            results,
        }
    }
}

// A multithreaded group, each bench has an individual app state
pub struct Independent<PARAM, MSG>
where PARAM: Send{
    pub id: usize,
    pub parameters: PARAM,
    pub setup: Box<dyn Fn(&ArchwayApp, &PARAM) -> Setup<MSG> + Send + Sync>,
}

impl<PARAM: Naming + Send + Sync, MSG: Sized + Serialize> Job for Independent<PARAM, MSG> {
    fn set_group_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_group_id(&self) -> usize {
        self.id
    }

    fn run(&self, app: ArchwayApp) -> JobResult {
        let bench_name = self.parameters.name();
        let tx_params = (self.setup)(&app, &self.parameters);
        JobResult {
            group_id: self.id,
            results: vec![bench_msg(&app, bench_name, tx_params)],
        }
    }
}

fn get_balance_as_aarch(bank: Bank<ArchwayApp>, addr: &SigningAccount) -> u128 {
    bank.query_balance(&QueryBalanceRequest {
        address: addr.address(),
        denom: FEE_DENOM.to_string(),
    })
    .ok()
    .map(|r| r.balance.map(|c| c.amount.parse::<u128>().unwrap()))
    .flatten()
    .unwrap_or(0)
}

fn bench_msg<MSG: Sized + Serialize>(
    app: &ArchwayApp,
    name: String,
    setup: Setup<MSG>,
) -> BenchResult {
    let initial_balance = get_balance_as_aarch(Bank::new(app), &setup.signer);

    let wasm = Wasm::new(app);
    let res = wasm
        .execute(&setup.contract, &setup.msg, &setup.funds, &setup.signer)
        .unwrap();

    BenchResult {
        name,
        gas: Gas {
            wanted: res.gas_info.gas_wanted as u128,
            used: res.gas_info.gas_used as u128,
        },
        arch: initial_balance - get_balance_as_aarch(Bank::new(app), &setup.signer),
    }
}
