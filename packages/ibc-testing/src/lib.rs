mod chain;
mod ibc_runtime;

pub use archway_rpc;
pub use ibc_runtime::*;

#[cfg(test)]
mod tests {
    use crate::IbcRuntimeBuilder;
    use archway_rpc::proto::any::Any;
    use archway_rpc::proto::cosmos::auth::v1beta1::BaseAccount;
    use archway_rpc::proto::cosmos::staking::v1beta1::MsgDelegate;
    use archway_rpc::proto::ibc::applications::interchain_accounts::v1::InterchainAccount;
    use archway_rpc::proto::prost::{Message, Name};
    use archway_rpc::{Auth, Authz, Bank, Staking, Timestamp, Wasm};
    use ica_demo::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn demo() {
        let wasm = std::fs::read("./../../artifacts/ica_demo.wasm").unwrap();

        let runtime = IbcRuntimeBuilder::new().build(None).await.unwrap();

        let mut deployer = runtime.chain1.new_account(100000).await.unwrap();

        // Deploy cosmwasm
        let store = runtime
            .chain1
            .client
            .store_code(&mut deployer, wasm, None)
            .await
            .unwrap();

        let addr = runtime
            .chain1
            .client
            .instantiate_contract(&mut deployer, store, &InstantiateMsg {}, None, None, vec![])
            .await
            .unwrap();

        sleep(Duration::from_secs(120)).await;

        let res = runtime
            .chain1
            .client
            .execute_contract(
                &deployer,
                addr.clone(),
                &ExecuteMsg::CreateICA {
                    connection_id: "connection-0".to_string(),
                },
                vec![],
            )
            .unwrap()
            .poll(&mut deployer)
            .await
            .unwrap();

        // dbg!(res.tx_result.events);
        sleep(Duration::from_secs(60)).await;

        // Get interchain account
        let mut ica = None;
        let mut tries = 6;
        while ica.is_none() && tries > 0 {
            // Get new accounts
            let res = runtime.chain2.client.accounts(None).await.unwrap();

            for account in res.value.accounts {
                if account
                    .type_url
                    .contains(InterchainAccount::<Vec<u8>>::NAME)
                {
                    let any: Any<InterchainAccount<BaseAccount<Vec<u8>>>> =
                        Any::decode(account.encode_to_vec().as_slice()).unwrap();
                    dbg!(&account);
                    ica = Some(any.value.base_account.unwrap().address);
                }
            }

            tries -= 1;
        }
        let ica = ica.unwrap();

        // Create user wallet
        let mut user = runtime.chain2.new_account(10000).await.unwrap();

        // TODO: give auth perms and fund with gas money
        runtime
            .chain2
            .client
            .grant_authorization(
                &user,
                ica.clone(),
                MsgDelegate::type_url(),
                Some(Timestamp {
                    seconds: 2364210061,
                    nanos: 0,
                }),
            )
            .unwrap()
            .poll(&mut user)
            .await
            .unwrap();

        let balance = runtime
            .chain2
            .client
            .balance(
                user.prefixed_pubkey().unwrap().to_string(),
                "stake".to_string(),
            )
            .await
            .unwrap()
            .value
            .balance
            .unwrap()
            .amount;

        // Get validator
        let res = runtime
            .chain2
            .client
            .validators(None, None)
            .await
            .unwrap()
            .value
            .validators;
        let validator_addr = res.first().unwrap().operator_address.to_string();

        // TODO: make auth command through smart contract
        let res = runtime
            .chain1
            .client
            .execute_contract(
                &deployer,
                addr.clone(),
                &ExecuteMsg::ExecuteICA {
                    connection_id: "connection-0".to_string(),
                    grantee: ica,
                    delegator: user.prefixed_pubkey().unwrap().to_string(),
                    validator: validator_addr,
                },
                vec![],
            )
            .unwrap()
            .poll(&mut deployer)
            .await
            .unwrap();

        sleep(Duration::from_secs(60)).await;
        let new_balance = runtime
            .chain2
            .client
            .balance(
                user.prefixed_pubkey().unwrap().to_string(),
                "stake".to_string(),
            )
            .await
            .unwrap()
            .value
            .balance
            .unwrap()
            .amount;
        println!("{balance} {new_balance}");
        assert_ne!(balance, new_balance);

        let res = runtime
            .chain1
            .client
            .query_contract(
                addr.clone(),
                &QueryMsg::IcaAccount {
                    connection_id: "connection-0".to_string(),
                },
            )
            .await
            .unwrap();

        dbg!(&res);

        runtime.stop().await;
    }
}
