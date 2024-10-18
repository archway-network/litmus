use crate::msg::SudoMsg;
use crate::state::{IcaAccount, ICA_ACCOUNTS, ICA_HISTORY, PENDING_ACCOUNT};
use crate::ContractError;
use archway_proto::any::Any;
use archway_proto::archway::cwerrors::v1::SudoError;
use archway_proto::archway::cwica::v1::{IcaSuccess, MsgRegisterInterchainAccount, MsgSendTx};
use archway_proto::cosmos::authz::v1beta1::MsgExec;
use archway_proto::cosmos::base::v1beta1::Coin;
use archway_proto::cosmos::staking::v1beta1::MsgDelegate;
use archway_proto::prost::{Message, Name};
use cosmwasm_std::{
    entry_point, to_json_binary, Attribute, Binary, CosmosMsg, DepsMut, Env, Response, SubMsg,
    Uint128,
};
// TODO: create a system based on handshakes

pub fn register(
    from_address: String,
    connection_id: String,
    response: &mut Response,
) -> Result<(), ContractError> {
    let regsiter_msg = MsgRegisterInterchainAccount {
        contract_address: from_address.clone(),
        connection_id: connection_id.clone(),
    };

    response
        .attributes
        .push(Attribute::new("action", "register"));
    response
        .attributes
        .push(Attribute::new("account_owner", from_address));
    response
        .attributes
        .push(Attribute::new("connection_id", connection_id));
    response.messages.push(SubMsg::new(CosmosMsg::Stargate {
        type_url: MsgRegisterInterchainAccount::type_url(),
        value: Binary(regsiter_msg.encode_to_vec()),
    }));
    Ok(())
}

pub fn execute_stake(
    env: &Env,
    connection_id: String,
    grantee: String,
    delegator: String,
    validator: String,
    amount: Uint128,
    response: &mut Response,
) -> Result<(), ContractError> {
    let execute_msg = MsgSendTx {
        contract_address: env.contract.address.to_string(),
        connection_id,
        msgs: vec![Any::new(MsgExec {
            grantee,
            msgs: vec![Any::new(MsgDelegate {
                delegator_address: delegator,
                validator_address: validator,
                amount: Some(Coin {
                    denom: "stake".to_string(),
                    amount: amount.to_string(),
                }),
            })],
        })],
        memo: "".to_string(),
        timeout: 5000,
    };

    response.messages.push(SubMsg::new(CosmosMsg::Stargate {
        type_url: MsgSendTx::<Vec<u8>>::type_url(),
        value: Binary(execute_msg.encode_to_vec()),
    }));

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::Ica(msg) => sudo_ica(deps, env, msg),
        SudoMsg::Error(msg) => sudo_error(deps, env, msg),
    }?;
    Ok(Response::new())
}

pub fn sudo_ica(deps: DepsMut, env: Env, msg: IcaSuccess) -> Result<Response, ContractError> {
    let response = Response::new();

    if let Some(ica) = msg.account_registered.as_ref() {
        if let Some((chain_name, delegator)) = PENDING_ACCOUNT.may_load(deps.storage)? {
            ICA_ACCOUNTS.save(
                deps.storage,
                chain_name,
                &IcaAccount {
                    ica_host_address: ica.counterparty_address.clone(),
                    delegator_address: delegator,
                },
            )?;
        } else {
            return Err(ContractError::NoPendingIca {});
        }
    }

    ICA_HISTORY.update::<_, ContractError>(deps.storage, |mut history| {
        history.push(msg);
        Ok(history)
    })?;

    Ok(response)
}

pub fn sudo_error(deps: DepsMut, env: Env, msg: SudoError) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("error_msg", to_json_binary(&msg)?.to_string()))
}
