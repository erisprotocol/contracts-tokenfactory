use std::{collections::HashSet, convert::TryFrom};

use cosmwasm_std::{
    Addr, Coin, Decimal, Env, QuerierWrapper, QueryRequest, StakingQuery, StdError, StdResult,
    Storage, Uint128, ValidatorResponse,
};
use eris::{
    governance_helper::get_period,
    helpers::bps::BasicPoints,
    hub::{DelegationStrategy, WantedDelegationsShare},
};
use eris_chain_adapter::types::main_denom;
use itertools::Itertools;

use crate::{state::State, types::Delegation};

/// Query the amounts of Luna a staker is delegating to a specific validator
pub(crate) fn query_delegation(
    querier: &QuerierWrapper,
    validator: &str,
    delegator_addr: &Addr,
) -> StdResult<Delegation> {
    Ok(Delegation {
        validator: validator.to_string(),
        amount: querier
            .query_delegation(delegator_addr, validator)?
            .map(|fd| fd.amount.amount.u128())
            .unwrap_or(0),
    })
}

/// Query the amounts of Luna a staker is delegating to each of the validators specified
pub(crate) fn query_delegations(
    querier: &QuerierWrapper,
    validators: &[String],
    delegator_addr: &Addr,
) -> StdResult<Vec<Delegation>> {
    validators
        .iter()
        .map(|validator| query_delegation(querier, validator, delegator_addr))
        .collect()
}

pub(crate) fn query_all_delegations(
    querier: &QuerierWrapper,
    delegator_addr: &Addr,
) -> StdResult<Vec<Delegation>> {
    let result: Vec<_> = querier
        .query_all_delegations(delegator_addr)?
        .into_iter()
        .filter(|d| d.amount.denom == main_denom() && !d.amount.amount.is_zero())
        .map(|d| Delegation {
            validator: d.validator,
            amount: d.amount.amount.u128(),
        })
        .collect();

    Ok(result)
}

/// Find the amount of a denom sent along a message, assert it is non-zero, and no other denom were
/// sent together
pub(crate) fn parse_received_fund(funds: &[Coin], denom: &str) -> StdResult<Uint128> {
    if funds.len() != 1 {
        return Err(StdError::generic_err(format!(
            "must deposit exactly one coin; received {}",
            funds.len()
        )));
    }

    let fund = &funds[0];
    if fund.denom != denom {
        return Err(StdError::generic_err(format!(
            "expected {} deposit, received {}",
            denom, fund.denom
        )));
    }

    if fund.amount.is_zero() {
        return Err(StdError::generic_err("deposit amount must be non-zero"));
    }

    Ok(fund.amount)
}

pub fn assert_validator_exists(querier: &QuerierWrapper, validator: &String) -> StdResult<()> {
    let _result: ValidatorResponse =
        querier.query(&QueryRequest::Staking(StakingQuery::Validator {
            address: validator.into(),
        }))?;
    Ok(())
}

pub fn assert_validators_exists(
    querier: &QuerierWrapper,
    validators: &Vec<String>,
) -> StdResult<()> {
    for validator in validators {
        assert_validator_exists(querier, validator)?;
    }
    Ok(())
}

/// Dedupes a Vector of strings using a hashset.
pub fn dedupe(validators: &mut Vec<String>) {
    let mut set = HashSet::new();

    validators.retain(|x| set.insert(x.clone()));
}

/// Calculates the wanted delegations based on the delegation strategy and the amp + emp gauges
/// The source of the gauges is flexible via the loader
/// This is only a read operation, so it can be used from queries aswell
pub(crate) fn get_wanted_delegations(
    state: &State,
    env: &Env,
    storage: &dyn Storage,
    _querier: &QuerierWrapper,
    // loader: impl GaugeLoader,
) -> StdResult<(WantedDelegationsShare, bool)> {
    let delegation_strategy =
        state.delegation_strategy.may_load(storage)?.unwrap_or(DelegationStrategy::Uniform {});

    match delegation_strategy {
        DelegationStrategy::Uniform {} => {
            let validators = state.validators.load(storage)?;
            let validator_count = Uint128::new(validators.len() as u128);
            let share_per_validator = Decimal::from_ratio(Uint128::one(), validator_count);

            Ok((
                WantedDelegationsShare {
                    tune_time: env.block.time.seconds(),
                    tune_period: get_period(env.block.time.seconds())?,
                    shares: validators
                        .into_iter()
                        .map(|val| (val, share_per_validator))
                        .collect_vec(),
                },
                // no need to store it
                false,
            ))
        },
        DelegationStrategy::Defined {
            shares_bps,
        } => Ok((
            WantedDelegationsShare {
                tune_time: env.block.time.seconds(),
                tune_period: get_period(env.block.time.seconds())?,
                shares: shares_bps
                    .into_iter()
                    .map(|(validator, bps)| Ok((validator, BasicPoints::try_from(bps)?.decimal())))
                    .collect::<StdResult<Vec<(String, Decimal)>>>()?,
            },
            // store it for get_utoken_per_validator
            true,
        )),
        // DelegationStrategy::Gauges {
        //     amp_gauges,
        //     emp_gauges,
        //     amp_factor_bps,
        //     min_delegation_bps,
        //     max_delegation_bps,
        //     validator_count,
        // } => {
        //     let min_delegation = BasicPoints::try_from(min_delegation_bps)?.decimal();
        //     let max_delegation = BasicPoints::try_from(max_delegation_bps)?.decimal();

        //     let vamp_factor = BasicPoints::try_from(amp_factor_bps)?.decimal();
        //     let emp_factor = Decimal::one().checked_sub(vamp_factor)?;

        //     let vamp_context = Context::from_amps(&loader, querier, amp_gauges)?;
        //     let emp_context = Context::from_emps(&loader, querier, emp_gauges)?;

        //     let validators: Vec<_> = state
        //         .validators
        //         .load(storage)?
        //         .into_iter()
        //         .map(|val| -> StdResult<(String, Decimal, Decimal)> {
        //             let vamp = vamp_context.points.get(&val).copied().unwrap_or_default();

        //             let total_share = if let Some(emp_context) = &emp_context {
        //                 let vamp_share =
        //                     vamp_factor.checked_mul(Decimal::from_ratio(vamp, vamp_context.sum))?;

        //                 let emp = emp_context.points.get(&val).copied().unwrap_or(Uint128::zero());
        //                 let emp_share =
        //                     emp_factor.checked_mul(Decimal::from_ratio(emp, emp_context.sum))?;

        //                 vamp_share.checked_add(emp_share)?
        //             } else {
        //                 Decimal::from_ratio(vamp, vamp_context.sum)
        //             };

        //             let score = Decimal::min(total_share, max_delegation);

        //             Ok((val, score, total_share))
        //         })
        //         .collect::<StdResult<Vec<_>>>()?
        //         .into_iter()
        //         .filter(|(_, amount, _)| *amount > min_delegation)
        //         .sorted_by(|(_, _, a), (_, _, b)| b.cmp(a)) // Sort in descending order
        //         .take(validator_count.into())
        //         .collect();

        //     // normalize missing percentage over all validators
        //     let total: Decimal = validators.iter().map(|a| a.1).sum();
        //     let validators: Vec<_> = validators
        //         .into_iter()
        //         .map(|v| -> StdResult<(String, Decimal)> {
        //             let normalized =
        //                 v.1.checked_div(total)
        //                     .map_err(|_| StdError::generic_err("Could not divide by total"))?;

        //             Ok((v.0, normalized))
        //         })
        //         .collect::<StdResult<Vec<_>>>()?;

        //     Ok((
        //         WantedDelegationsShare {
        //             shares: validators,
        //             tune_time: env.block.time.seconds(),
        //             tune_period: get_period(env.block.time.seconds())?,
        //         },
        //         true,
        //     ))
        // },
    }
}

// struct Context {
//     pub sum: Uint128,
//     pub points: HashMap<String, Uint128>,
// }

// impl Context {
//     pub fn from_emps(
//         loader: &impl GaugeLoader,
//         querier: &QuerierWrapper,
//         emp_gauges: Option<Addr>,
//     ) -> StdResult<Option<Context>> {
//         if let Some(emp_gauges) = emp_gauges {
//             let emp_info = loader.get_emp_tune_info(querier, emp_gauges)?;
//             let emp_sum: Uint128 = emp_info.emp_points.iter().map(|a| a.1).sum();
//             let emp_points: HashMap<_, _> =
//                 emp_info.emp_points.into_iter().map(|v| (v.0.to_string(), v.1)).collect();

//             if emp_sum.is_zero() {
//                 return Err(StdError::generic_err("EMP not tuned."));
//             }

//             Ok(Some(Self {
//                 sum: emp_sum,
//                 points: emp_points,
//             }))
//         } else {
//             Ok(None)
//         }
//     }

//     pub fn from_amps(
//         loader: &impl GaugeLoader,
//         querier: &QuerierWrapper,
//         amp_gauges: Addr,
//     ) -> StdResult<Context> {
//         let vamp_info = loader.get_amp_tune_info(querier, amp_gauges)?;
//         let vamp_sum: Uint128 = vamp_info.vamp_points.iter().map(|a| a.1).sum();
//         let vamp_points: HashMap<_, _> =
//             vamp_info.vamp_points.into_iter().map(|v| (v.0.to_string(), v.1)).collect();

//         if vamp_sum.is_zero() {
//             return Err(StdError::generic_err("No vAMP. Vote first before tuning."));
//         }

//         Ok(Self {
//             sum: vamp_sum,
//             points: vamp_points,
//         })
//     }
// }
