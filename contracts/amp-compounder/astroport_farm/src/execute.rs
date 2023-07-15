use astroport::asset::token_asset_info;
use cosmwasm_std::{DepsMut, Response, StdError, StdResult, SubMsgResponse, Uint128};

use crate::state::{State, STATE};

pub fn register_amp_lp_token(deps: DepsMut, response: SubMsgResponse) -> StdResult<Response> {
    let event = response
        .events
        .iter()
        .find(|event| event.ty == "instantiate")
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let contract_addr_str = &event
        .attributes
        .iter()
        .find(|attr| attr.key == "_contract_address" || attr.key == "_contract_addr")
        .ok_or_else(|| StdError::generic_err("cannot find `_contract_address` attribute"))?
        .value;

    let contract_addr = deps.api.addr_validate(contract_addr_str)?;

    STATE.save(
        deps.storage,
        &State {
            amp_lp_token: token_asset_info(contract_addr),
            total_bond_share: Uint128::zero(),
        },
    )?;

    Ok(Response::new())
}
