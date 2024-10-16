use cosmwasm_std::{Binary, Deps, DepsMut, entry_point, Env, MessageInfo, Reply, Response, StdResult, SubMsg, SubMsgResult, to_json_binary};
use cw_storage_plus::Item;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut res = Response::new();
    match msg {
        ExecuteMsg::CosmosMsg(cosmos_msg) => {
            res.messages.push(SubMsg::new(cosmos_msg));
        }
        ExecuteMsg::Loop(n) => {
            let storage = Item::new("demo");
            for i in 0..n {
                storage.save(deps.storage, &i.to_string())?;
            }
        }
    }
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::from(&[]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(_) => Ok(Response::default()),
        SubMsgResult::Err(_) => Err(ContractError::Unauthorized {}),
    }
}
