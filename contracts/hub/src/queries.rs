use cosmwasm_std::{Addr, Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

// use eris::governance_helper::get_period;
use eris::hub::{
    Batch, ConfigResponse, PendingBatch, StateResponse, UnbondRequestsByBatchResponseItem,
    UnbondRequestsByUserResponseItem, UnbondRequestsByUserResponseItemDetails,
    WantedDelegationsResponse,
};
use eris_chain_adapter::types::main_denom;
use itertools::Itertools;

use crate::helpers::{get_wanted_delegations, query_delegations};
use crate::math::get_utoken_per_validator_prepared;
use crate::state::State;
// use crate::types::gauges::PeriodGaugeLoader;

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = State::default();

    Ok(ConfigResponse {
        owner: state.owner.load(deps.storage)?.into(),
        operator: state.operator.load(deps.storage)?.into(),
        new_owner: state.new_owner.may_load(deps.storage)?.map(|addr| addr.into()),
        stake_token: state.stake_token.load(deps.storage)?.denom,
        epoch_period: state.epoch_period.load(deps.storage)?,
        unbond_period: state.unbond_period.load(deps.storage)?,
        validators: state.validators.load(deps.storage)?,
        fee_config: state.fee_config.load(deps.storage)?,
        stages_preset: state.stages_preset.may_load(deps.storage)?.unwrap_or_default(),
        withdrawls_preset: state.withdrawls_preset.may_load(deps.storage)?.unwrap_or_default(),
        allow_donations: state.allow_donations.may_load(deps.storage)?.unwrap_or(false),
        delegation_strategy: match state
            .delegation_strategy
            .may_load(deps.storage)?
            .unwrap_or(eris::hub::DelegationStrategy::Uniform)
        {
            eris::hub::DelegationStrategy::Uniform => eris::hub::DelegationStrategy::Uniform,
            eris::hub::DelegationStrategy::Defined {
                shares_bps,
            } => eris::hub::DelegationStrategy::Defined {
                shares_bps,
            },
            // eris::hub::DelegationStrategy::Gauges {
            //     amp_gauges,
            //     emp_gauges,
            //     amp_factor_bps,
            //     min_delegation_bps,
            //     max_delegation_bps,
            //     validator_count,
            // } => eris::hub::DelegationStrategy::Gauges {
            //     amp_gauges: amp_gauges.to_string(),
            //     emp_gauges: emp_gauges.map(|a| a.to_string()),
            //     amp_factor_bps,
            //     min_delegation_bps,
            //     max_delegation_bps,
            //     validator_count,
            // },
        },
        vote_operator: state.vote_operator.may_load(deps.storage)?.map(|addr| addr.into()),
    })
}

pub fn state(deps: Deps, env: Env) -> StdResult<StateResponse> {
    let state = State::default();

    let stake_token = state.stake_token.load(deps.storage)?;
    let total_ustake = stake_token.total_supply;

    let validators = state.validators.load(deps.storage)?;
    let delegations = query_delegations(&deps.querier, &validators, &env.contract.address)?;
    let total_utoken: u128 = delegations.iter().map(|d| d.amount).sum();

    // only not reconciled batches are relevant as they are still unbonding and estimated unbond time in the future.
    let unbonding: u128 = state
        .previous_batches
        .idx
        .reconciled
        .prefix(false.into())
        .range(deps.storage, None, None, Order::Descending)
        .map(|item| {
            let (_, v) = item.unwrap();
            v
        })
        .filter(|item| item.est_unbond_end_time > env.block.time.seconds())
        .map(|item| item.utoken_unclaimed.u128())
        .sum();

    let available = deps.querier.query_balance(&env.contract.address, main_denom())?.amount;

    let exchange_rate = if total_ustake.is_zero() {
        Decimal::one()
    } else {
        Decimal::from_ratio(total_utoken, total_ustake)
    };

    Ok(StateResponse {
        total_ustake,
        total_utoken: Uint128::new(total_utoken),
        exchange_rate,
        unlocked_coins: state.unlocked_coins.load(deps.storage)?,
        unbonding: Uint128::from(unbonding),
        available,
        tvl_utoken: Uint128::from(total_utoken)
            .checked_add(Uint128::from(unbonding))?
            .checked_add(available)?,
    })
}

pub fn wanted_delegations(deps: Deps, env: Env) -> StdResult<WantedDelegationsResponse> {
    let state = State::default();

    let (delegations, _, _, share) = get_utoken_per_validator_prepared(
        &state,
        deps.storage,
        &deps.querier,
        &env.contract.address,
        None,
    )?;

    Ok(WantedDelegationsResponse {
        delegations: sort_delegations(delegations),
        tune_time_period: share.map(|s| (s.tune_time, s.tune_period)),
    })
}

pub fn simulate_wanted_delegations(
    deps: Deps,
    env: Env,
    _period: Option<u64>,
) -> StdResult<WantedDelegationsResponse> {
    let state = State::default();

    // let period = period.unwrap_or(get_period(env.block.time.seconds())? + 1);

    let (delegation_goal, _) = get_wanted_delegations(
        &state,
        &env,
        deps.storage,
        &deps.querier,
        // PeriodGaugeLoader {
        //     period,
        // },
    )?;

    let (delegations, _, _, share) = get_utoken_per_validator_prepared(
        &state,
        deps.storage,
        &deps.querier,
        &env.contract.address,
        Some(delegation_goal),
    )?;

    Ok(WantedDelegationsResponse {
        delegations: sort_delegations(delegations), // Sort in descending order
        tune_time_period: share.map(|s| (s.tune_time, s.tune_period)),
    })
}
/// Sort delegations by amount descending and then by address ascending
fn sort_delegations(
    delegations: std::collections::HashMap<String, Uint128>,
) -> Vec<(String, Uint128)> {
    delegations
        .into_iter()
        .sorted_by(|(vala, a), (valb, b)| {
            let result = b.cmp(a);
            if result == std::cmp::Ordering::Equal {
                return vala.cmp(valb);
            }
            result
        })
        .collect()
}

pub fn pending_batch(deps: Deps) -> StdResult<PendingBatch> {
    let state = State::default();
    state.pending_batch.load(deps.storage)
}

pub fn previous_batch(deps: Deps, id: u64) -> StdResult<Batch> {
    let state = State::default();
    state.previous_batches.load(deps.storage, id)
}

pub fn previous_batches(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<Batch>> {
    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    state
        .previous_batches
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect()
}

pub fn unbond_requests_by_batch(
    deps: Deps,
    id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<UnbondRequestsByBatchResponseItem>> {
    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let mut start: Option<Bound<&Addr>> = None;
    let addr: Addr;
    if let Some(start_after) = start_after {
        if let Ok(start_after_addr) = deps.api.addr_validate(&start_after) {
            addr = start_after_addr;
            start = Some(Bound::exclusive(&addr));
        }
    }

    state
        .unbond_requests
        .prefix(id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;
            Ok(v.into())
        })
        .collect()
}

pub fn unbond_requests_by_user(
    deps: Deps,
    user: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<UnbondRequestsByUserResponseItem>> {
    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = deps.api.addr_validate(&user)?;
    let start = start_after.map(|id| Bound::exclusive((id, &addr)));

    state
        .unbond_requests
        .idx
        .user
        .prefix(user)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;

            Ok(v.into())
        })
        .collect()
}

pub fn unbond_requests_by_user_details(
    deps: Deps,
    user: String,
    start_after: Option<u64>,
    limit: Option<u32>,
    env: Env,
) -> StdResult<Vec<UnbondRequestsByUserResponseItemDetails>> {
    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = deps.api.addr_validate(&user)?;
    let start = start_after.map(|id| Bound::exclusive((id, &addr)));

    let pending = state.pending_batch.load(deps.storage)?;

    state
        .unbond_requests
        .idx
        .user
        .prefix(user)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, v) = item?;

            let state_msg: String;
            let previous: Option<Batch>;
            if pending.id == v.id {
                state_msg = "PENDING".to_string();
                previous = None;
            } else {
                let batch = state.previous_batches.load(deps.storage, v.id)?;
                previous = Some(batch.clone());
                let current_time = env.block.time.seconds();
                state_msg = if batch.est_unbond_end_time < current_time {
                    "COMPLETED".to_string()
                } else {
                    "UNBONDING".to_string()
                }
            }

            Ok(UnbondRequestsByUserResponseItemDetails {
                id: v.id,
                shares: v.shares,
                state: state_msg,
                pending: if pending.id == v.id {
                    Some(pending.clone())
                } else {
                    None
                },
                batch: previous,
            })
        })
        .collect()
}
