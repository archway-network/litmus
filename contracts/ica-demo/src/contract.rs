#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::ICA_HISTORY;
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
        ExecuteMsg::CreateICA { connection_id } => {
            let mut response = Response::new();
            register(
                env.contract.address.to_string(),
                connection_id,
                &mut response,
            )?;
            Ok(response)
        }
        ExecuteMsg::ExecuteICA {
            connection_id,
            grantee,
            delegator,
            validator,
        } => {
            let mut response = Response::new();

            execute_stake(
                &env,
                connection_id,
                grantee,
                delegator,
                validator,
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
