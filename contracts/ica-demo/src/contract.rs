#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ICA_ACCOUNTS, ICA_HISTORY, PENDING_ACCOUNT};
use crate::sudo::{execute_stake, register};
/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ica-demo";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ICA_HISTORY.save(deps.storage, &vec![])?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateICA {
            chain_name,
            delegator_address,
            connection_id,
        } => {
            let mut response = Response::new();

            if PENDING_ACCOUNT.exists(deps.storage) {
                return Err(ContractError::PendingIca {});
            }

            PENDING_ACCOUNT.save(deps.storage, &(chain_name, delegator_address))?;

            register(
                env.contract.address.to_string(),
                connection_id,
                &mut response,
            )?;
            Ok(response)
        }
        ExecuteMsg::InitDelegation {
            connection_id,
            chain_name,
            validator,
            amount,
        } => {
            let mut response = Response::new();

            let ica = ICA_ACCOUNTS.load(deps.storage, chain_name)?;

            execute_stake(
                &env,
                connection_id,
                ica.ica_host_address,
                ica.delegator_address,
                validator,
                amount,
                &mut response,
            )?;

            Ok(response)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(match msg {
        QueryMsg::IcaAccount { .. } => to_json_binary(&ICA_HISTORY.load(deps.storage)?)?,
    })
}
