use crate::{
    constants::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractResult,
    extensions::UtilizationMethodEx,
    state::{BalanceLocked, State},
};
use cosmwasm_std::{Addr, DepsMut, Env, Response, StdResult, Uint128};
use cw2::set_contract_version;
use eris::arb_vault::{InstantiateMsg, LpToken, LsdConfig, ValidatedConfig};
use eris_chain_adapter::types::chain;
use eris_chain_shared::chain_trait::ChainInterface;

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

pub fn instantiate(deps: DepsMut, env: Env, msg: InstantiateMsg) -> ContractResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State::default();
    let chain = chain(&env);

    let lsds = msg
        .lsds
        .into_iter()
        .map(|lsd| lsd.validate(deps.api))
        .collect::<StdResult<Vec<LsdConfig<Addr>>>>()?;

    msg.utilization_method.validate()?;

    let config = ValidatedConfig {
        unbond_time_s: msg.unbond_time_s,
        lsds,
        utoken: msg.utoken,
        utilization_method: msg.utilization_method,
    };
    state.config.save(deps.storage, &config)?;
    state.owner.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;
    state.unbond_id.save(deps.storage, &0)?;
    state.fee_config.save(deps.storage, &msg.fee_config.validate(deps.api)?)?;

    state.update_whitelist(deps.storage, deps.api, msg.whitelist)?;

    state.balance_locked.save(
        deps.storage,
        &BalanceLocked {
            balance: Uint128::zero(),
        },
    )?;

    let sub_denom = msg.denom;
    let full_denom = chain.get_token_denom(env.contract.address, sub_denom.clone());

    state.lp_token.save(
        deps.storage,
        &LpToken {
            denom: full_denom.clone(),
            total_supply: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_message(chain.create_denom_msg(full_denom, sub_denom)))
}
