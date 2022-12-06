use std::ops::Add;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Order};
use cw_storage_plus::Bound;

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, EntryResponse, ListResponse};
use crate::state::{Config, ENTRY_SEQ, LIST, CONFIG, Entry, Priority, Status};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cargo-generated-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);
    // If the instantiation message contains an owner address, validate the address and use it.
    // Otherwise, the owner is the address that instantiates the contract.    

    let config = Config {
        owner: owner.clone()
    };
    // Save the owner address to contract storage.
    CONFIG.save(deps.storage, &config)?;
    // Save the entry sequence to contract storage, starting from 0.
    ENTRY_SEQ.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::NewEntry {description, priority} => execute_create_new_entry(deps, info, description, priority),
        ExecuteMsg::UpdateEntry {id, description, status, priority } => execute_update_entry(deps, info, id, description, status, priority),
        ExecuteMsg::DeleteEntry {id} => execute_delete_entry(deps, info, id)
    }
}

pub fn execute_create_new_entry(deps: DepsMut, info: MessageInfo, description: String, priority: Option<Priority>) -> Result<Response, ContractError> {
    // Before creating the new entry, the function checks if the message sender is 
    // the owner of the contract.
    let owner = CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        // If not, it returns an error and the new entry creation fails to be performed.
        return Err(ContractError::Unauthorized {});
    }
    // In order to generate a unique `id` for the new entry, the function increments the entry sequence 
    // and saves it to the contract storage with `ENTRY_SEQ.update()`.
    let id = ENTRY_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| {
        Ok(id.add(1))
    })?;
    /*
       The new entry is defined with the received `description` and `priority` attributes. 
       The `status` of the new entry is set to `ToDo` by default. Notice that `priority` 
       is an optional parameter. If not provided, the 'priority' will be set as `None` by default.
    */
    let new_entry = Entry {
        id,
        description,
        priority: priority.unwrap_or(Priority::None),
        status: Status::ToDo
    };
    // The function finally saves the new entry to the `LIST` with the matching `id` and 
    // returns a `Response` with the relevant attributes. 
    LIST.save(deps.storage, id, &new_entry)?;
    Ok(Response::new().add_attribute("method", "execute_create_new_entry")
        .add_attribute("new_entry_id", id.to_string()))
}

pub fn execute_update_entry(deps: DepsMut, info: MessageInfo, id: u64, description: Option<String>, status: Option<Status>, priority: Option<Priority>) -> Result<Response, ContractError> {
    // Before continuing with the new update, the function checks if the message sender is 
    // the owner of the contract.
    let owner = CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        // If not, it returns an error and the update fails to be performed.
        return Err(ContractError::Unauthorized {});
    }
    // The entry with the matching `id` is loaded from the `LIST`.
    let entry = LIST.load(deps.storage, id)?;
    /*
       Sharing the same id, an updated version of the entry is defined with the received values for 
       `description`, `status` and `priority`. These are optional parameters and if any one of them is not 
       provided, the function defaults back to the corresponding value from the entry loaded.
    */
    let updated_entry = Entry {
        id,
        description: description.unwrap_or(entry.description),
        status: status.unwrap_or(entry.status),
        priority: priority.unwrap_or(entry.priority),
    };
    // The function saves the updated entry to the `LIST` with the matching `id` and returns a `Response` 
    // with the relevant attributes.
    LIST.save(deps.storage, id, &updated_entry)?;
    Ok(Response::new().add_attribute("method", "execute_update_entry")
                      .add_attribute("updated_entry_id", id.to_string()))
}

pub fn execute_delete_entry(deps: DepsMut, info: MessageInfo, id: u64) -> Result<Response, ContractError> {
    // Before carrying on with the removal, the function checks if the message sender is 
    // the owner of the contract.
    let owner = CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        // If not, it returns an error and the deletion fails to be performed.
        return Err(ContractError::Unauthorized {});
    }
    // The entry with the matching `id` is removed from the `LIST`.
    LIST.remove(deps.storage, id);
    // The function returns a `Response` with the relevant attributes.
    Ok(Response::new().add_attribute("method", "execute_delete_entry")
                      .add_attribute("deleted_entry_id", id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryEntry{ id } => to_binary(&query_entry(deps, id)?),
        QueryMsg::QueryList { start_after, limit } => to_binary(&query_list(deps, start_after, limit)?),
    }}

pub fn query_entry(deps: Deps, id: u64) -> StdResult<EntryResponse> {
    let entry = LIST.load(deps.storage, id)?;
    Ok(EntryResponse { id: id, description: entry.description, status: entry.status, priority: entry.priority })
}

pub fn query_list(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<ListResponse> {
    
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let entries: StdResult<Vec<_>> = LIST
    .range(deps.storage, start, None, Order::Ascending)
    .take(limit)
    .collect();
    
    let result = ListResponse {
        entries: entries?.into_iter().map(|l| l.1.into()).collect(),
    };
    Ok(result)
}

#[cfg(test)]
mod tests {}
