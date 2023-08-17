use crate::error::{ContractError, CustomResult};
use crate::extensions::{BalancesEx, ConfigEx};
use crate::helpers::calc_fees;
use crate::state::{State, UnbondHistory};
use astroport::asset::native_asset_info;
use cosmwasm_std::{Decimal, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;
use eris::arb_vault::{
    BalancesOptionalDetails, ConfigResponse, ExchangeHistory, ExchangeRatesResponse, StateDetails,
    StateResponse, TakeableResponse, UnbondItem, UnbondRequestsResponse, UserInfoResponse,
};
use eris::constants::DAY;
use eris::voting_escrow::{DEFAULT_LIMIT, MAX_LIMIT};
use std::ops::Div;

pub fn query_config(deps: Deps) -> CustomResult<ConfigResponse> {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let lp_token = state.lp_token.load(deps.storage)?;

    let fee_config = state.fee_config.load(deps.storage)?;
    let owner = state.owner.load(deps.storage)?;
    let whitelist = state.whitelisted_addrs.may_load(deps.storage)?;
    Ok(ConfigResponse {
        config,
        owner,
        fee_config,
        whitelist,
        lp_token,
    })
}

pub fn query_takeable(
    deps: Deps,
    env: Env,
    wanted_profit: Option<Decimal>,
) -> CustomResult<TakeableResponse> {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let mut lsds = config.lsd_group(&env);
    let balances = lsds.get_total_assets_err(deps, &env, &state, &config)?;

    Ok(TakeableResponse {
        takeable: match wanted_profit {
            Some(wanted) => Some(balances.calc_takeable_for_profit(&config, &wanted)?),
            _ => None,
        },
        steps: balances.calc_all_takeable_steps(&config).map_err(|e| {
            ContractError::CalculationError("takeable for steps".into(), e.to_string())
        })?,
    })
}

pub fn query_unbond_requests(
    deps: Deps,
    env: Env,
    address: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> CustomResult<UnbondRequestsResponse> {
    let address = deps.api.addr_validate(&address)?;
    let state = State::default();
    let fee_config = state.fee_config.load(deps.storage)?;

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start: Option<Bound<u64>> = start_after.map(Bound::exclusive);

    let unbond_history = state
        .unbond_history
        .prefix(address)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<(u64, UnbondHistory)>>>()?;

    let current_time = env.block.time.seconds();

    Ok(UnbondRequestsResponse {
        requests: unbond_history
            .into_iter()
            .map(|(id, item)| {
                let withdraw_pool_fee_factor = item.pool_fee_factor(current_time);

                let (withdraw_protocol_fee, withdraw_pool_fee) =
                    calc_fees(&fee_config, item.amount_asset, withdraw_pool_fee_factor).map_err(
                        |e| ContractError::CalculationError("fees".into(), e.to_string()),
                    )?;

                Ok(UnbondItem {
                    id,
                    released: item.release_time <= current_time,
                    start_time: item.start_time,
                    release_time: item.release_time,
                    amount_asset: item.amount_asset,
                    withdraw_protocol_fee,
                    withdraw_pool_fee,
                })
            })
            .collect::<CustomResult<Vec<UnbondItem>>>()?,
    })
}

pub fn query_state(
    deps: Deps,
    env: Env,
    include_details: Option<bool>,
) -> CustomResult<StateResponse> {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let lp_token = state.lp_token.load(deps.storage)?;
    let mut lsds = config.lsd_group(&env);

    let total_lp_supply: Uint128 = lp_token.total_supply;
    let balances = lsds.get_total_assets_err(deps, &env, &state, &config)?;
    let details = if include_details.unwrap_or_default() {
        Some(StateDetails {
            takeable_steps: balances.calc_all_takeable_steps(&config).map_err(|e| {
                ContractError::CalculationError("takeable for steps".into(), e.to_string())
            })?,
        })
    } else {
        None
    };

    let resp = StateResponse {
        exchange_rate: if total_lp_supply.is_zero() {
            Decimal::one()
        } else {
            Decimal::from_ratio(balances.vault_total, total_lp_supply)
        },
        total_lp_supply,
        balances: BalancesOptionalDetails {
            tvl_utoken: balances.tvl_utoken,
            vault_total: balances.vault_total,
            vault_available: balances.vault_available,
            vault_takeable: balances.vault_takeable,
            locked_user_withdrawls: balances.locked_user_withdrawls,
            lsd_unbonding: balances.lsd_unbonding,
            lsd_withdrawable: balances.lsd_withdrawable,
            lsd_xvalue: balances.lsd_xvalue,
            details: if include_details.unwrap_or_default() {
                Some(balances.details)
            } else {
                None
            },
        },
        details,
    };

    Ok(resp)
}

pub fn query_user_info(deps: Deps, env: Env, address: String) -> CustomResult<UserInfoResponse> {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let lp_token = state.lp_token.load(deps.storage)?;

    let mut lsds = config.lsd_group(&env);
    let address = deps.api.addr_validate(&address)?;

    let total_lp_supply = lp_token.total_supply;
    let balances = lsds.get_total_assets_err(deps, &env, &state, &config)?;

    let lp_amount = native_asset_info(lp_token.denom).query_pool(&deps.querier, address)?;

    let utoken_amount = if total_lp_supply.is_zero() {
        Uint128::zero()
    } else {
        lp_amount.multiply_ratio(balances.vault_total, total_lp_supply)
    };

    Ok(UserInfoResponse {
        utoken_amount,
        lp_amount,
    })
}

pub fn query_exchange_rates(
    deps: Deps,
    _env: Env,
    start_after_d: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ExchangeRatesResponse> {
    let state = State::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let end = start_after_d.map(Bound::exclusive);
    let exchange_rates = state
        .exchange_history
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .collect::<StdResult<Vec<(u64, ExchangeHistory)>>>()?;

    let apr: Option<Decimal> = if exchange_rates.len() > 1 {
        let (_, current) = exchange_rates.first().unwrap();
        let (_, last) = exchange_rates.last().unwrap();

        let delta_time_s = current.time_s - last.time_s;
        let delta_rate = current.exchange_rate.checked_sub(last.exchange_rate).unwrap_or_default();

        Some(
            delta_rate
                .checked_mul(Decimal::from_ratio(DAY, delta_time_s).div(last.exchange_rate))?,
        )
    } else {
        None
    };

    Ok(ExchangeRatesResponse {
        exchange_rates,
        apr,
    })
}
