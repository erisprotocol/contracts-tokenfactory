use cosmwasm_std::{
    attr, to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, DistributionMsg,
    Env, Event, Order, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use eris::helper::validate_received_funds;
use eris::{CustomEvent, CustomResponse, DecimalCheckedOps};

use eris::hub::{
    Batch, CallbackMsg, DelegationStrategy, ExecuteMsg, FeeConfig, InstantiateMsg, PendingBatch,
    SingleSwapConfig, StakeToken, UnbondRequest,
};
use eris_chain_adapter::types::{
    chain, get_balances_hashmap, CustomMsgType, DenomType, HubChainConfigInput, WithdrawType,
};
use itertools::Itertools;

use crate::constants::get_reward_fee_cap;
use crate::error::{ContractError, ContractResult};
use crate::helpers::{
    assert_validator_exists, assert_validators_exists, dedupe, get_wanted_delegations,
    query_all_delegations, query_all_delegations_amount, query_delegation, query_delegations,
};
use crate::math::{
    compute_mint_amount, compute_redelegations_for_rebalancing, compute_redelegations_for_removal,
    compute_unbond_amount, compute_undelegations, mark_reconciled_batches, reconcile_batches,
};
use crate::state::State;
use crate::types::gauges::TuneInfoGaugeLoader;
// use crate::types::gauges::TuneInfoGaugeLoader;
use crate::types::{Coins, Delegation, SendFee};

use eris_chain_shared::chain_trait::{ChainInterface, Validateable};

const CONTRACT_NAME: &str = "eris-hub";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

//--------------------------------------------------------------------------------------------------
// Instantiation
//--------------------------------------------------------------------------------------------------

pub fn instantiate(deps: DepsMut, env: Env, msg: InstantiateMsg) -> ContractResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State::default();
    let chain = chain(&env);

    if msg.protocol_reward_fee.gt(&get_reward_fee_cap()) {
        return Err(ContractError::ProtocolRewardFeeTooHigh {});
    }

    if msg.epoch_period == 0 {
        return Err(ContractError::CantBeZero("epoch_period".into()));
    }

    if msg.unbond_period == 0 {
        return Err(ContractError::CantBeZero("unbond_period".into()));
    }

    state.owner.save(deps.storage, &deps.api.addr_validate(&msg.owner)?)?;
    state.operator.save(deps.storage, &deps.api.addr_validate(&msg.operator)?)?;
    state.epoch_period.save(deps.storage, &msg.epoch_period)?;
    state.unbond_period.save(deps.storage, &msg.unbond_period)?;

    if let Some(vote_operator) = msg.vote_operator {
        state.vote_operator.save(deps.storage, &deps.api.addr_validate(&vote_operator)?)?;
    }

    // by default donations are set to false
    state.allow_donations.save(deps.storage, &false)?;

    let mut validators = msg.validators;

    dedupe(&mut validators);
    assert_validators_exists(&deps.querier, &validators)?;

    state.validators.save(deps.storage, &validators)?;
    state.unlocked_coins.save(deps.storage, &vec![])?;
    state.fee_config.save(
        deps.storage,
        &FeeConfig {
            protocol_fee_contract: deps.api.addr_validate(&msg.protocol_fee_contract)?,
            protocol_reward_fee: msg.protocol_reward_fee,
        },
    )?;

    state.pending_batch.save(
        deps.storage,
        &PendingBatch {
            id: 1,
            ustake_to_burn: Uint128::zero(),
            est_unbond_start_time: env.block.time.seconds() + msg.epoch_period,
        },
    )?;

    let delegation_strategy = msg.delegation_strategy.unwrap_or(DelegationStrategy::Uniform);
    state
        .delegation_strategy
        .save(deps.storage, &delegation_strategy.validate(deps.api, &validators)?)?;

    state.chain_config.save(deps.storage, &msg.chain_config.validate(deps.api)?)?;

    let sub_denom = msg.denom;
    let full_denom = chain.get_token_denom(env.contract.address, sub_denom.clone());
    state.stake_token.save(
        deps.storage,
        &StakeToken {
            utoken: msg.utoken,
            denom: full_denom.clone(),
            total_supply: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_message(chain.create_denom_msg(full_denom, sub_denom)))
}

//--------------------------------------------------------------------------------------------------
// Bonding and harvesting logics
//--------------------------------------------------------------------------------------------------

/// NOTE: In a previous implementation, we split up the deposited Token over all validators, so that
/// they all have the same amount of delegation. This is however quite gas-expensive: $1.5 cost in
/// the case of 15 validators.
///
/// To save gas for users, now we simply delegate all deposited Token to the validator with the
/// smallest amount of delegation. If delegations become severely unbalance as a result of this
/// (e.g. when a single user makes a very big deposit), anyone can invoke `ExecuteMsg::Rebalance`
/// to balance the delegations.
pub fn bond(
    deps: DepsMut,
    env: Env,
    receiver: Addr,
    funds: &[Coin],
    donate: bool,
) -> ContractResult {
    let state = State::default();
    let mut stake = state.stake_token.load(deps.storage)?;

    let token_to_bond = validate_received_funds(funds, &stake.utoken)?;

    let (new_delegation, delegations) =
        find_new_delegation(&state, &deps, &env, token_to_bond, &stake.utoken)?;

    // Query the current supply of Staking Token and compute the amount to mint
    let ustake_supply = stake.total_supply;
    let ustake_to_mint = if donate {
        match state.allow_donations.may_load(deps.storage)? {
            Some(false) => Err(ContractError::DonationsDisabled {})?,
            Some(true) | None => {
                // if it is not set (backward compatibility) or set to true, donations are allowed
            },
        }
        Uint128::zero()
    } else {
        compute_mint_amount(ustake_supply, token_to_bond, &delegations)
    };

    let event = Event::new("erishub/bonded")
        .add_attribute("receiver", receiver.clone())
        .add_attribute("token_bonded", token_to_bond)
        .add_attribute("ustake_minted", ustake_to_mint);

    let mint_msgs: Option<Vec<CosmosMsg<CustomMsgType>>> = if donate {
        None
    } else {
        // create mint message and add to stored total supply
        stake.total_supply = stake.total_supply.checked_add(ustake_to_mint)?;
        state.stake_token.save(deps.storage, &stake)?;

        Some(chain(&env).create_mint_msgs(stake.denom.clone(), ustake_to_mint, receiver))
    };

    Ok(Response::new()
        .add_message(new_delegation.to_cosmos_msg())
        .add_optional_messages(mint_msgs)
        .add_message(check_received_coin_msg(&deps, &env, stake, Some(token_to_bond))?)
        .add_event(event)
        .add_attribute("action", "erishub/bond"))
}

pub fn harvest(
    deps: DepsMut,
    env: Env,
    withdrawals: Option<Vec<(WithdrawType, DenomType)>>,
    stages: Option<Vec<Vec<SingleSwapConfig>>>,
    sender: Addr,
) -> ContractResult {
    let state = State::default();
    let stake = state.stake_token.load(deps.storage)?;

    // 1. Withdraw delegation rewards
    let withdraw_submsgs: Vec<CosmosMsg<CustomMsgType>> =
        query_all_delegations(&deps.querier, &env.contract.address, &stake.utoken)?
            .into_iter()
            .map(|d| {
                CosmosMsg::Distribution(DistributionMsg::WithdrawDelegatorReward {
                    validator: d.validator,
                })
            })
            .collect::<Vec<_>>();

    // 2. Prepare LP withdrawals / deconstruction
    let withdrawals =
        state.get_or_preset(deps.storage, withdrawals, &state.withdrawals_preset, &sender)?;
    let withdrawal_msg = withdrawals.map(|withdrawals| CallbackMsg::WithdrawLps {
        withdrawals,
    });

    // 3. Prepare swap stages
    let stages = state.get_or_preset(deps.storage, stages, &state.stages_preset, &sender)?;
    validate_no_utoken_or_ustake_swap(&stages, &stake)?;
    let swap_msgs = stages.map(|stages| {
        stages
            .into_iter()
            .map(|stage| CallbackMsg::SingleStageSwap {
                stage,
            })
            .collect_vec()
    });

    Ok(Response::new()
        // 1. Withdraw delegation rewards
        .add_messages(withdraw_submsgs)
        // 2. Withdraw / Destruct LPs
        .add_optional_callback(&env, withdrawal_msg)?
        // 3 swap - multiple single stage swaps
        .add_optional_callbacks(&env, swap_msgs)?
        // 4. apply received total utoken to unlocked_coins
        .add_message(check_received_coin_msg(
            &deps,
            &env,
            state.stake_token.load(deps.storage)?,
            None,
        )?)
        // 5. restake unlocked_coins
        .add_callback(&env, CallbackMsg::Reinvest {})?
        .add_attribute("action", "erishub/harvest"))
}

/// this method will split LP positions into each single position
pub fn withdraw_lps(
    deps: DepsMut,
    env: Env,
    withdrawals: Vec<(WithdrawType, DenomType)>,
) -> ContractResult {
    let mut withdraw_msgs: Vec<CosmosMsg<CustomMsgType>> = vec![];
    let chain = chain(&env);
    let get_denoms = || withdrawals.iter().map(|a| a.1.clone()).collect_vec();
    let balances = get_balances_hashmap(&deps, env, get_denoms)?;
    let get_chain_config = || State::default().chain_config.load(deps.storage);

    for (withdraw_type, denom) in withdrawals {
        let balance = balances.get(&denom.to_string());

        if let Some(balance) = balance {
            if !balance.is_zero() {
                let msg =
                    chain.create_withdraw_msg(get_chain_config, withdraw_type, denom, *balance)?;
                if let Some(msg) = msg {
                    withdraw_msgs.push(msg);
                }
            }
        }
    }

    Ok(Response::new().add_messages(withdraw_msgs).add_attribute("action", "erishub/withdraw_lps"))
}

/// swaps all unlocked coins to token
pub fn single_stage_swap(deps: DepsMut, env: Env, stage: Vec<SingleSwapConfig>) -> ContractResult {
    let state = State::default();
    let chain = chain(&env);
    let default_max_spread = state.get_default_max_spread(deps.storage);
    let get_chain_config = || state.chain_config.load(deps.storage);
    let get_denoms = || stage.iter().map(|a| a.1.clone()).collect_vec();
    let balances = get_balances_hashmap(&deps, env, get_denoms)?;

    let mut response = Response::new().add_attribute("action", "erishub/single_stage_swap");
    // iterate all specified swaps of the stage
    for (stage_type, denom, belief_price) in stage {
        let balance = balances.get(&denom.to_string());
        // check if the swap also has a balance in the contract
        if let Some(balance) = balance {
            if !balance.is_zero() {
                // create a single swap message add add to submsgs
                let msg = chain.create_single_stage_swap_msgs(
                    get_chain_config,
                    stage_type,
                    denom,
                    *balance,
                    belief_price,
                    default_max_spread,
                )?;
                response = response.add_message(msg)
            }
        }
    }

    Ok(response)
}

fn validate_no_utoken_or_ustake_swap(
    stages: &Option<Vec<Vec<SingleSwapConfig>>>,
    stake_token: &StakeToken,
) -> Result<(), ContractError> {
    if let Some(stages) = stages {
        for stage in stages {
            for (_addr, denom, _) in stage {
                if denom.to_string() == stake_token.utoken || denom.to_string() == stake_token.denom
                {
                    return Err(ContractError::SwapFromNotAllowed(denom.to_string()));
                }
            }
        }
    }
    Ok(())
}

fn validate_no_belief_price(stages: &Vec<Vec<SingleSwapConfig>>) -> Result<(), ContractError> {
    for stage in stages {
        for (_, _, belief_price) in stage {
            if belief_price.is_some() {
                return Err(ContractError::BeliefPriceNotAllowed {});
            }
        }
    }
    Ok(())
}

/// This callback is used to take a current snapshot of the balance and add the received balance to the unlocked_coins state after the execution
fn check_received_coin_msg(
    deps: &DepsMut,
    env: &Env,
    stake: StakeToken,
    // offset to account for funds being sent that should be ignored
    negative_offset: Option<Uint128>,
) -> StdResult<CosmosMsg<CustomMsgType>> {
    let mut amount =
        deps.querier.query_balance(env.contract.address.to_string(), &stake.utoken)?.amount;

    if let Some(negative_offset) = negative_offset {
        amount = amount.checked_sub(negative_offset)?;
    }

    let amount_stake =
        deps.querier.query_balance(env.contract.address.to_string(), stake.denom.clone())?.amount;

    CallbackMsg::CheckReceivedCoin {
        // 0. take current balance - offset
        snapshot: Coin {
            denom: stake.utoken,
            amount,
        },
        snapshot_stake: Coin {
            denom: stake.denom,
            amount: amount_stake,
        },
    }
    .into_cosmos_msg(&env.contract.address)
}

/// NOTE:
/// 1. When delegation Token here, we don't need to use a `SubMsg` to handle the received coins,
/// because we have already withdrawn all claimable staking rewards previously in the same atomic
/// execution.
/// 2. Same as with `bond`, in the latest implementation we only delegate staking rewards with the
/// validator that has the smallest delegation amount.
pub fn reinvest(deps: DepsMut, env: Env) -> ContractResult {
    let state = State::default();
    let fee_config = state.fee_config.load(deps.storage)?;
    let mut unlocked_coins = state.unlocked_coins.load(deps.storage)?;
    let mut stake = state.stake_token.load(deps.storage)?;

    if unlocked_coins.is_empty() {
        return Err(ContractError::NoTokensAvailable(format!(
            "{0}, {1}",
            stake.utoken, stake.denom
        )));
    }

    let mut event = Event::new("erishub/harvested");
    let mut msgs: Vec<CosmosMsg<CustomMsgType>> = vec![];

    let mut total_utoken: Option<u128> = None;

    for coin in unlocked_coins.iter() {
        let available = coin.amount;
        let protocol_fee = fee_config.protocol_reward_fee.checked_mul_uint(available)?;
        let remaining = available.saturating_sub(protocol_fee);

        let send_fee = if coin.denom == stake.utoken {
            let to_bond = remaining;
            // if receiving normal utoken -> restake
            let (new_delegation, delegations) =
                find_new_delegation(&state, &deps, &env, to_bond, &stake.utoken)?;

            let utoken_staked: u128 = delegations.iter().map(|d| d.amount).sum();
            total_utoken = Some(utoken_staked + to_bond.u128());

            event = event
                .add_attribute("utoken_bonded", to_bond)
                .add_attribute("utoken_protocol_fee", protocol_fee);

            msgs.push(new_delegation.to_cosmos_msg());
            true
        } else if coin.denom == stake.denom {
            // if receiving ustake (staked utoken) -> burn
            event = event
                .add_attribute("ustake_burned", remaining)
                .add_attribute("ustake_protocol_fee", protocol_fee);

            stake.total_supply = stake.total_supply.checked_sub(remaining)?;
            state.stake_token.save(deps.storage, &stake)?;
            msgs.push(chain(&env).create_burn_msg(stake.denom.clone(), remaining));
            true
        } else {
            // we can ignore other coins as we will only store utoken and ustake there
            false
        };

        if send_fee && !protocol_fee.is_zero() {
            let send_fee = SendFee::new(
                fee_config.protocol_fee_contract.clone(),
                protocol_fee.u128(),
                coin.denom.clone(),
            );
            msgs.push(send_fee.to_cosmos_msg());
        }
    }

    // remove the converted coins. Unlocked_coins track utoken ([TOKEN]) and ustake (amp[TOKEN]).
    unlocked_coins.retain(|coin| coin.denom != stake.utoken && coin.denom != stake.denom);
    state.unlocked_coins.save(deps.storage, &unlocked_coins)?;

    // update exchange_rate history
    let exchange_rate = calc_current_exchange_rate(total_utoken, &deps, &env, stake)?;
    state.exchange_history.save(deps.storage, env.block.time.seconds(), &exchange_rate)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_event(event)
        .add_attribute("action", "erishub/reinvest")
        .add_attribute("exchange_rate", exchange_rate.to_string()))
}

fn calc_current_exchange_rate(
    total_utoken: Option<u128>,
    deps: &DepsMut,
    env: &Env,
    stake: StakeToken,
) -> Result<Decimal, ContractError> {
    let total_utoken = match total_utoken {
        Some(val) => val,
        None => query_all_delegations_amount(&deps.querier, &env.contract.address, &stake.utoken)?,
    };
    let exchange_rate = if stake.total_supply.is_zero() {
        Decimal::one()
    } else {
        Decimal::from_ratio(total_utoken, stake.total_supply)
    };
    Ok(exchange_rate)
}

pub fn callback_received_coins(
    deps: DepsMut,
    env: Env,
    snapshot: Coin,
    snapshot_stake: Coin,
) -> ContractResult {
    let state = State::default();
    // in some cosmwasm versions the events are not received in the callback
    // so each time the contract can receive some coins from rewards we also need to check after receiving some and add them to the unlocked_coins

    let mut received_coins = Coins(vec![]);
    let mut event = Event::new("erishub/received");

    event = event.add_optional_attribute(add_to_received_coins(
        &deps,
        env.contract.address.clone(),
        snapshot,
        &mut received_coins,
    )?);

    event = event.add_optional_attribute(add_to_received_coins(
        &deps,
        env.contract.address,
        snapshot_stake,
        &mut received_coins,
    )?);

    if !received_coins.0.is_empty() {
        state.unlocked_coins.update(deps.storage, |coins| -> StdResult<_> {
            let mut coins = Coins(coins);
            coins.add_many(&received_coins)?;
            Ok(coins.0)
        })?;
    }

    Ok(Response::new().add_event(event).add_attribute("action", "erishub/received"))
}

fn add_to_received_coins(
    deps: &DepsMut,
    contract: Addr,
    snapshot: Coin,
    received_coins: &mut Coins,
) -> Result<Option<Attribute>, ContractError> {
    let current_balance = deps.querier.query_balance(contract, snapshot.denom.to_string())?.amount;

    let attr = if current_balance > snapshot.amount {
        let received_amount = current_balance.checked_sub(snapshot.amount)?;
        let received = Coin::new(received_amount.u128(), snapshot.denom);
        received_coins.add(&received)?;
        Some(attr("received_coin", received.to_string()))
    } else {
        None
    };

    Ok(attr)
}

/// searches for the validator with the least amount of delegations
/// For Uniform mode, searches through the validators list
/// For Gauge mode, searches for all delegations, and if nothing found, use the first validator from the list.
fn find_new_delegation(
    state: &State,
    deps: &DepsMut,
    env: &Env,
    utoken_to_bond: Uint128,
    utoken: &String,
) -> Result<(Delegation, Vec<Delegation>), ContractError> {
    let delegation_strategy =
        state.delegation_strategy.may_load(deps.storage)?.unwrap_or(DelegationStrategy::Uniform {});

    let delegations = match delegation_strategy {
        DelegationStrategy::Uniform {} => {
            let validators = state.validators.load(deps.storage)?;
            query_delegations(&deps.querier, &validators, &env.contract.address)?
        },
        DelegationStrategy::Gauges {
            ..
        }
        | DelegationStrategy::Defined {
            ..
        } => {
            // if we have gauges, only delegate to validators that have delegations, all others are "inactive"
            let mut delegations =
                query_all_delegations(&deps.querier, &env.contract.address, utoken)?;
            if delegations.is_empty() {
                let validators = state.validators.load(deps.storage)?;

                if let Some(first_validator) = validators.first() {
                    delegations = vec![Delegation {
                        amount: 0,
                        validator: first_validator.to_string(),
                        denom: utoken.clone(),
                    }]
                } else {
                    return Err(ContractError::NoValidatorsConfigured);
                }
            }
            delegations
        },
    };

    // Query the current delegations made to validators, and find the validator with the smallest
    // delegated amount through a linear search
    // The code for linear search is a bit uglier than using `sort_by` but cheaper: O(n) vs O(n * log(n))
    let mut validator = &delegations[0].validator;
    let mut amount = delegations[0].amount;

    for d in &delegations[1..] {
        if d.amount < amount {
            validator = &d.validator;
            amount = d.amount;
        }
    }
    let new_delegation = Delegation::new(validator, utoken_to_bond.u128(), utoken);

    Ok((new_delegation, delegations))
}

//--------------------------------------------------------------------------------------------------
// Unbonding logics
//--------------------------------------------------------------------------------------------------

pub fn queue_unbond(
    deps: DepsMut,
    env: Env,
    receiver: Addr,
    ustake_to_burn: Uint128,
) -> ContractResult {
    let state = State::default();

    let mut pending_batch = state.pending_batch.load(deps.storage)?;
    pending_batch.ustake_to_burn += ustake_to_burn;
    state.pending_batch.save(deps.storage, &pending_batch)?;

    state.unbond_requests.update(
        deps.storage,
        (pending_batch.id, &receiver),
        |x| -> StdResult<_> {
            let mut request = x.unwrap_or_else(|| UnbondRequest {
                id: pending_batch.id,
                user: receiver.clone(),
                shares: Uint128::zero(),
            });
            request.shares += ustake_to_burn;
            Ok(request)
        },
    )?;

    let mut msgs: Vec<CosmosMsg<CustomMsgType>> = vec![];
    let mut start_time = pending_batch.est_unbond_start_time.to_string();
    if env.block.time.seconds() > pending_batch.est_unbond_start_time {
        start_time = "immediate".to_string();
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.into(),
            msg: to_binary(&ExecuteMsg::SubmitBatch {})?,
            funds: vec![],
        }));
    }

    let event = Event::new("erishub/unbond_queued")
        .add_attribute("est_unbond_start_time", start_time)
        .add_attribute("id", pending_batch.id.to_string())
        .add_attribute("receiver", receiver)
        .add_attribute("ustake_to_burn", ustake_to_burn);

    Ok(Response::new()
        .add_messages(msgs)
        .add_event(event)
        .add_attribute("action", "erishub/queue_unbond"))
}

pub fn submit_batch(deps: DepsMut, env: Env) -> ContractResult {
    let state = State::default();
    let mut stake = state.stake_token.load(deps.storage)?;
    let validators = state.validators.load(deps.storage)?;
    let unbond_period = state.unbond_period.load(deps.storage)?;
    let pending_batch = state.pending_batch.load(deps.storage)?;

    let current_time = env.block.time.seconds();
    if current_time < pending_batch.est_unbond_start_time {
        return Err(ContractError::SubmitBatchAfter(pending_batch.est_unbond_start_time));
    }

    let delegations = query_all_delegations(&deps.querier, &env.contract.address, &stake.utoken)?;
    let ustake_supply = stake.total_supply;

    let utoken_to_unbond =
        compute_unbond_amount(ustake_supply, pending_batch.ustake_to_burn, &delegations);
    let new_undelegations = compute_undelegations(
        &state,
        deps.storage,
        utoken_to_unbond,
        &delegations,
        validators,
        &stake.utoken,
    )?;

    state.previous_batches.save(
        deps.storage,
        pending_batch.id,
        &Batch {
            id: pending_batch.id,
            reconciled: false,
            total_shares: pending_batch.ustake_to_burn,
            utoken_unclaimed: utoken_to_unbond,
            est_unbond_end_time: current_time + unbond_period,
        },
    )?;

    let epoch_period = state.epoch_period.load(deps.storage)?;
    state.pending_batch.save(
        deps.storage,
        &PendingBatch {
            id: pending_batch.id + 1,
            ustake_to_burn: Uint128::zero(),
            est_unbond_start_time: current_time + epoch_period,
        },
    )?;

    let undelegate_msgs =
        new_undelegations.into_iter().map(|d| d.to_cosmos_msg()).collect::<Vec<_>>();

    // apply burn to the stored total supply and save state
    stake.total_supply = stake.total_supply.checked_sub(pending_batch.ustake_to_burn)?;
    state.stake_token.save(deps.storage, &stake)?;
    let burn_msg: CosmosMsg<CustomMsgType> =
        chain(&env).create_burn_msg(stake.denom.clone(), pending_batch.ustake_to_burn);

    let event = Event::new("erishub/unbond_submitted")
        .add_attribute("id", pending_batch.id.to_string())
        .add_attribute("utoken_unbonded", utoken_to_unbond)
        .add_attribute("ustake_burned", pending_batch.ustake_to_burn);

    Ok(Response::new()
        .add_messages(undelegate_msgs)
        .add_message(burn_msg)
        .add_message(check_received_coin_msg(&deps, &env, stake, None)?)
        .add_event(event)
        .add_attribute("action", "erishub/unbond"))
}

pub fn reconcile(deps: DepsMut, env: Env) -> ContractResult {
    let state = State::default();
    let stake = state.stake_token.load(deps.storage)?;
    let current_time = env.block.time.seconds();

    // Load batches that have not been reconciled
    let all_batches = state
        .previous_batches
        .idx
        .reconciled
        .prefix(false.into())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect::<StdResult<Vec<_>>>()?;

    let mut batches = all_batches
        .into_iter()
        .filter(|b| current_time > b.est_unbond_end_time)
        .collect::<Vec<_>>();

    let utoken_expected_received: Uint128 = batches.iter().map(|b| b.utoken_unclaimed).sum();

    if utoken_expected_received.is_zero() {
        return Ok(Response::new());
    }
    let unlocked_coins = state.unlocked_coins.load(deps.storage)?;
    let utoken_expected_unlocked = Coins(unlocked_coins).find(&stake.utoken).amount;

    let utoken_expected = utoken_expected_received + utoken_expected_unlocked;
    let utoken_actual = deps.querier.query_balance(&env.contract.address, stake.utoken)?.amount;

    if utoken_actual >= utoken_expected {
        mark_reconciled_batches(&mut batches);
        for batch in &batches {
            state.previous_batches.save(deps.storage, batch.id, batch)?;
        }
        let ids = batches.iter().map(|b| b.id.to_string()).collect::<Vec<_>>().join(",");
        let event = Event::new("erishub/reconciled")
            .add_attribute("ids", ids)
            .add_attribute("utoken_deducted", "0");
        return Ok(Response::new().add_event(event).add_attribute("action", "erishub/reconcile"));
    }

    let utoken_to_deduct = utoken_expected - utoken_actual;

    let reconcile_info = reconcile_batches(&mut batches, utoken_to_deduct);

    for batch in &batches {
        state.previous_batches.save(deps.storage, batch.id, batch)?;
    }

    let ids = batches.iter().map(|b| b.id.to_string()).collect::<Vec<_>>().join(",");

    let event = Event::new("erishub/reconciled")
        .add_attribute("ids", ids)
        .add_attribute("utoken_deducted", utoken_to_deduct.to_string())
        .add_optional_attribute(reconcile_info);

    Ok(Response::new().add_event(event).add_attribute("action", "erishub/reconcile"))
}

pub fn withdraw_unbonded(deps: DepsMut, env: Env, user: Addr, receiver: Addr) -> ContractResult {
    let state = State::default();
    let current_time = env.block.time.seconds();

    // NOTE: If the user has too many unclaimed requests, this may not fit in the WASM memory...
    // However, this is practically never going to happen. Who would create hundreds of unbonding
    // requests and never claim them?
    let requests = state
        .unbond_requests
        .idx
        .user
        .prefix(user.to_string())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, v) = item?;
            Ok(v)
        })
        .collect::<StdResult<Vec<_>>>()?;

    // NOTE: Token in the following batches are withdrawn it the batch:
    // - is a _previous_ batch, not a _pending_ batch
    // - is reconciled
    // - has finished unbonding
    // If not sure whether the batches have been reconciled, the user should first invoke `ExecuteMsg::Reconcile`
    // before withdrawing.
    let mut total_utoken_to_refund = Uint128::zero();
    let mut ids: Vec<String> = vec![];
    for request in &requests {
        if let Ok(mut batch) = state.previous_batches.load(deps.storage, request.id) {
            if batch.reconciled && batch.est_unbond_end_time < current_time {
                let utoken_to_refund =
                    batch.utoken_unclaimed.multiply_ratio(request.shares, batch.total_shares);

                ids.push(request.id.to_string());

                total_utoken_to_refund += utoken_to_refund;
                batch.total_shares -= request.shares;
                batch.utoken_unclaimed -= utoken_to_refund;

                if batch.total_shares.is_zero() {
                    state.previous_batches.remove(deps.storage, request.id)?;
                } else {
                    state.previous_batches.save(deps.storage, batch.id, &batch)?;
                }

                state.unbond_requests.remove(deps.storage, (request.id, &user))?;
            }
        }
    }

    if total_utoken_to_refund.is_zero() {
        return Err(ContractError::CantBeZero("withdrawable amount".into()));
    }
    let stake = state.stake_token.load(deps.storage)?;

    let refund_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: receiver.clone().into(),
        amount: vec![Coin::new(total_utoken_to_refund.u128(), stake.utoken)],
    });

    let event = Event::new("erishub/unbonded_withdrawn")
        .add_attribute("ids", ids.join(","))
        .add_attribute("user", user)
        .add_attribute("receiver", receiver)
        .add_attribute("utoken_refunded", total_utoken_to_refund);

    Ok(Response::new()
        .add_message(refund_msg)
        .add_event(event)
        .add_attribute("action", "erishub/withdraw_unbonded"))
}

pub fn tune_delegations(deps: DepsMut, env: Env, sender: Addr) -> ContractResult {
    let state = State::default();
    state.assert_owner(deps.storage, &sender)?;
    let (wanted_delegations, save) =
        get_wanted_delegations(&state, &env, deps.storage, &deps.querier, TuneInfoGaugeLoader {})?;
    let attributes = if save {
        state.delegation_goal.save(deps.storage, &wanted_delegations)?;
        wanted_delegations
            .shares
            .iter()
            .map(|a| attr("goal_delegation", format!("{0}={1}", a.0, a.1)))
            .collect()
    } else {
        state.delegation_goal.remove(deps.storage);
        // these would be boring, as all are the same
        vec![]
    };
    Ok(Response::new()
        .add_attribute("action", "erishub/tune_delegations")
        .add_attributes(attributes))
}

//--------------------------------------------------------------------------------------------------
// Ownership and management logics
//--------------------------------------------------------------------------------------------------

pub fn rebalance(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    min_redelegation: Option<Uint128>,
) -> ContractResult {
    let state = State::default();
    let stake = state.stake_token.load(deps.storage)?;
    state.assert_owner(deps.storage, &sender)?;
    let validators = state.validators.load(deps.storage)?;
    let delegations = query_all_delegations(&deps.querier, &env.contract.address, &stake.utoken)?;

    let min_redelegation = min_redelegation.unwrap_or_default();

    let new_redelegations = compute_redelegations_for_rebalancing(
        &state,
        deps.storage,
        &delegations,
        validators,
        &stake.utoken,
    )?
    .into_iter()
    .filter(|redelegation| redelegation.amount >= min_redelegation.u128())
    .collect::<Vec<_>>();

    let redelegate_msgs = new_redelegations.iter().map(|rd| rd.to_cosmos_msg()).collect::<Vec<_>>();

    let amount: u128 = new_redelegations.iter().map(|rd| rd.amount).sum();

    let event = Event::new("erishub/rebalanced").add_attribute("utoken_moved", amount.to_string());

    let check_msg = if !redelegate_msgs.is_empty() {
        // only check coins if a redelegation is happening
        Some(check_received_coin_msg(&deps, &env, stake, None)?)
    } else {
        None
    };

    Ok(Response::new()
        .add_messages(redelegate_msgs)
        .add_optional_message(check_msg)
        .add_event(event)
        .add_attribute("action", "erishub/rebalance"))
}

pub fn add_validator(deps: DepsMut, sender: Addr, validator: String) -> ContractResult {
    let state = State::default();

    state.assert_owner(deps.storage, &sender)?;
    assert_validator_exists(&deps.querier, &validator)?;

    state.validators.update(deps.storage, |mut validators| {
        if validators.contains(&validator) {
            return Err(ContractError::ValidatorAlreadyWhitelisted(validator.clone()));
        }
        validators.push(validator.clone());
        Ok(validators)
    })?;

    let event = Event::new("erishub/validator_added").add_attribute("validator", validator);

    Ok(Response::new().add_event(event).add_attribute("action", "erishub/add_validator"))
}

pub fn remove_validator(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    validator: String,
) -> ContractResult {
    let state = State::default();
    let stake_token = state.stake_token.load(deps.storage)?;

    state.assert_owner(deps.storage, &sender)?;

    let validators = state.validators.update(deps.storage, |mut validators| {
        if !validators.contains(&validator) {
            return Err(ContractError::ValidatorNotWhitelisted(validator.clone()));
        }
        validators.retain(|v| *v != validator);

        if validators.is_empty() {
            return Err(ContractError::NoValidatorsConfigured);
        }

        Ok(validators)
    })?;

    let delegation_strategy =
        state.delegation_strategy.may_load(deps.storage)?.unwrap_or(DelegationStrategy::Uniform);

    let redelegate_msgs = match delegation_strategy {
        DelegationStrategy::Uniform => {
            // only redelegate when old strategy
            let delegations = query_delegations(&deps.querier, &validators, &env.contract.address)?;
            let delegation_to_remove =
                query_delegation(&deps.querier, &validator, &env.contract.address)?;
            let new_redelegations = compute_redelegations_for_removal(
                &state,
                deps.storage,
                &delegation_to_remove,
                &delegations,
                validators,
                &stake_token.utoken,
            )?;

            new_redelegations.iter().map(|d| d.to_cosmos_msg()).collect::<Vec<_>>()
        },
        DelegationStrategy::Gauges {
            ..
        }
        | DelegationStrategy::Defined {
            ..
        } => {
            // removed validators can have a delegation until the next tune, to keep undelegations in sync.
            vec![]
        },
    };

    let event = Event::new("erishub/validator_removed").add_attribute("validator", validator);

    let check_msg = if !redelegate_msgs.is_empty() {
        // only check coins if a redelegation is happening
        Some(check_received_coin_msg(&deps, &env, state.stake_token.load(deps.storage)?, None)?)
    } else {
        None
    };

    Ok(Response::new()
        .add_messages(redelegate_msgs)
        .add_optional_message(check_msg)
        .add_event(event)
        .add_attribute("action", "erishub/remove_validator"))
}

pub fn transfer_ownership(deps: DepsMut, sender: Addr, new_owner: String) -> ContractResult {
    let state = State::default();

    state.assert_owner(deps.storage, &sender)?;
    state.new_owner.save(deps.storage, &deps.api.addr_validate(&new_owner)?)?;

    Ok(Response::new().add_attribute("action", "erishub/transfer_ownership"))
}

pub fn drop_ownership_proposal(deps: DepsMut, sender: Addr) -> ContractResult {
    let state = State::default();

    state.assert_owner(deps.storage, &sender)?;
    state.new_owner.remove(deps.storage);

    Ok(Response::new().add_attribute("action", "erishub/drop_ownership_proposal"))
}

pub fn accept_ownership(deps: DepsMut, sender: Addr) -> ContractResult {
    let state = State::default();

    let previous_owner = state.owner.load(deps.storage)?;
    let new_owner = state.new_owner.load(deps.storage)?;

    if sender != new_owner {
        return Err(ContractError::UnauthorizedSenderNotNewOwner {});
    }

    state.owner.save(deps.storage, &sender)?;
    state.new_owner.remove(deps.storage);

    let event = Event::new("erishub/ownership_transferred")
        .add_attribute("new_owner", new_owner)
        .add_attribute("previous_owner", previous_owner);

    Ok(Response::new().add_event(event).add_attribute("action", "erishub/transfer_ownership"))
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    sender: Addr,
    protocol_fee_contract: Option<String>,
    protocol_reward_fee: Option<Decimal>,
    operator: Option<String>,
    stages_preset: Option<Vec<Vec<SingleSwapConfig>>>,
    withdrawals_preset: Option<Vec<(WithdrawType, DenomType)>>,
    allow_donations: Option<bool>,
    delegation_strategy: Option<DelegationStrategy>,
    vote_operator: Option<String>,
    chain_config: Option<HubChainConfigInput>,
    default_max_spread: Option<u64>,
) -> ContractResult {
    let state = State::default();

    state.assert_owner(deps.storage, &sender)?;

    if protocol_fee_contract.is_some() || protocol_reward_fee.is_some() {
        let mut fee_config = state.fee_config.load(deps.storage)?;

        if let Some(protocol_fee_contract) = protocol_fee_contract {
            fee_config.protocol_fee_contract = deps.api.addr_validate(&protocol_fee_contract)?;
        }

        if let Some(protocol_reward_fee) = protocol_reward_fee {
            if protocol_reward_fee.gt(&get_reward_fee_cap()) {
                return Err(ContractError::ProtocolRewardFeeTooHigh {});
            }
            fee_config.protocol_reward_fee = protocol_reward_fee;
        }

        state.fee_config.save(deps.storage, &fee_config)?;
    }

    if let Some(operator) = operator {
        state.operator.save(deps.storage, &deps.api.addr_validate(operator.as_str())?)?;
    }

    if let Some(chain_config) = chain_config {
        state.chain_config.save(deps.storage, &chain_config.validate(deps.api)?)?;
    }

    if stages_preset.is_some() {
        validate_no_utoken_or_ustake_swap(&stages_preset, &state.stake_token.load(deps.storage)?)?;
    }

    if let Some(stages_preset) = stages_preset {
        // belief price is not allowed. We still store it with None, as otherwise a lot of additional logic is required to load it.
        validate_no_belief_price(&stages_preset)?;
        state.stages_preset.save(deps.storage, &stages_preset)?;
    }

    if let Some(withdrawals_preset) = withdrawals_preset {
        state.withdrawals_preset.save(deps.storage, &withdrawals_preset)?;
    }

    if let Some(delegation_strategy) = delegation_strategy {
        let validators = state.validators.load(deps.storage)?;
        state
            .delegation_strategy
            .save(deps.storage, &delegation_strategy.validate(deps.api, &validators)?)?;
    }

    if let Some(allow_donations) = allow_donations {
        state.allow_donations.save(deps.storage, &allow_donations)?;
    }
    if let Some(default_max_spread) = default_max_spread {
        state.default_max_spread.save(deps.storage, &default_max_spread)?;
    }

    if let Some(vote_operator) = vote_operator {
        state.vote_operator.save(deps.storage, &deps.api.addr_validate(&vote_operator)?)?;
    }

    Ok(Response::new().add_attribute("action", "erishub/update_config"))
}
