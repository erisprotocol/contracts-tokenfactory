use crate::asserts::{assert_has_funds, assert_max_amount, assert_min_profit};
use crate::error::{ContractError, ContractResult};
use crate::extensions::{BalancesEx, ConfigEx};
use crate::helpers::{calc_fees, get_share_from_deposit};
use crate::state::{BalanceCheckpoint, BalanceLocked, State, UnbondHistory};
use astroport::asset::{native_asset, native_asset_info, Asset, AssetInfo};
use cosmwasm_std::{
    attr, coin, Addr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order, QuerierWrapper,
    Response, StdResult, Storage, Uint128, WasmMsg,
};
use eris::arb_vault::{CallbackMsg, ExecuteSubMsg, LpToken, ValidatedConfig};
use eris::{CustomMsgExt, CustomResponse};
use eris_chain_adapter::types::{chain, CustomMsgType};
use eris_chain_shared::chain_trait::ChainInterface;
use itertools::Itertools;
use std::vec;

//----------------------------------------------------------------------------------------
//  EXECUTE FUNCTIONS
//----------------------------------------------------------------------------------------

pub fn execute_arbitrage(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    message: ExecuteSubMsg,
    result_token: AssetInfo,
    wanted_profit: Decimal,
) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let mut lsds = config.lsd_group(&env);
    let balances = lsds.get_total_assets_err(deps.as_ref(), &env, &state, &config)?;

    let lsd = lsds.get_adapter_by_asset(result_token.clone())?;
    lsd.assert_not_disabled()?;
    state.assert_sender_whitelisted(deps.storage, &info.sender)?;
    state.assert_not_nested(deps.storage)?;
    assert_has_funds(&message.funds_amount)?;
    assert_min_profit(&wanted_profit)?;
    assert_max_amount(&config, &balances, &wanted_profit, &message.funds_amount)?;

    // setup contract to call, by default the sender is called with the funds requested
    let contract_addr = if let Some(contract_addr) = message.contract_addr {
        deps.api.addr_validate(&contract_addr)?
    } else {
        info.sender
    };
    let active_balance = balances.get_by_name(&lsd.name)?.clone();
    lsds.assert_not_lsd_contract(&contract_addr)?;

    // create balance checkpoint with total value, as it needs to be higher after full execution.
    state.balance_checkpoint.save(
        deps.storage,
        &BalanceCheckpoint {
            vault_available: balances.vault_available,
            tvl_utoken: balances.tvl_utoken,
            active_balance,
        },
    )?;

    let execute_flashloan: CosmosMsg<CustomMsgType> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: message.msg.clone(),
        funds: vec![Coin {
            denom: config.utoken,
            amount: message.funds_amount,
        }],
    });

    let validate_flashloan_result = CallbackMsg::AssertResult {
        result_token,
        wanted_profit,
    }
    .into_cosmos_msg(&env.contract.address)?;

    Ok(Response::new()
        .add_message(execute_flashloan)
        .add_message(validate_flashloan_result)
        .add_attribute("action", "arb/execute_arbitrage"))
}

pub fn execute_withdraw_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    names: Option<Vec<String>>,
) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let mut lsds = config.lsd_group_by_names(&env, names);

    state.assert_not_nested(deps.storage)?;
    state.assert_sender_whitelisted(deps.storage, &info.sender)?;

    let (messages, attributes) = lsds.get_withdraw_msgs(&deps)?;

    if messages.is_empty() {
        return Err(ContractError::NothingToWithdraw {});
    }

    Ok(Response::new().add_messages(messages).add_attributes(attributes))
}

pub fn execute_unbond_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    names: Option<Vec<String>>,
) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let mut lsds = config.lsd_group_by_names(&env, names);

    state.assert_not_nested(deps.storage)?;
    state.assert_sender_whitelisted(deps.storage, &info.sender)?;

    let (messages, attributes) = lsds.get_unbond_msgs(&deps)?;

    if messages.is_empty() {
        return Err(ContractError::NothingToUnbond {});
    }

    Ok(Response::new().add_messages(messages).add_attributes(attributes))
}

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deposit: Asset,
    recipient: Option<String>,
) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let mut lp_token = state.lp_token.load(deps.storage)?;
    let mut lsds = config.lsd_group(&env);

    state.assert_not_nested(deps.storage)?;
    deposit.info.check(deps.api)?;
    deposit.assert_sent_native_token_balance(&info)?;

    if deposit.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    if deposit.info != native_asset_info(config.utoken.clone()) {
        return Err(ContractError::AssetMismatch {});
    }

    let deposit_amount = deposit.amount;

    let assets = lsds.get_total_assets_err(deps.as_ref(), &env, &state, &config)?;

    // removing the deposit amount for correct share calculation
    let vault_utoken = assets.vault_total.checked_sub(deposit_amount)?;
    let share = get_share_from_deposit(lp_token.total_supply, vault_utoken, deposit_amount)?;

    // Mint LP tokens for the sender or for the receiver (if set)
    let recipient = if let Some(recipient) = recipient {
        deps.api.addr_validate(&recipient)?
    } else {
        info.sender.clone()
    };

    Ok(Response::new()
        .add_messages(create_mint_msgs(
            &env,
            deps.storage,
            &state,
            &mut lp_token,
            recipient.clone(),
            share,
        )?)
        .add_attributes(vec![
            attr("action", "arb/execute_deposit"),
            attr("sender", info.sender.to_string()),
            attr("recipient", recipient.to_string()),
            attr("deposit_amount", deposit_amount),
            attr("share", share.to_string()),
            attr("vault_utoken_new", assets.vault_total),
        ]))
}

pub fn execute_unbond_user(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    immediate: Option<bool>,
) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    let mut lp_token = state.lp_token.load(deps.storage)?;
    let mut lsds = config.lsd_group(&env);

    if info.funds.len() != 1 {
        return Err(ContractError::InvalidFunds {});
    }

    let default_fund = coin(0, lp_token.denom.clone());
    let fund = info.funds.first().unwrap_or(&default_fund);
    if fund.denom != lp_token.denom || fund.amount.is_zero() {
        return Err(ContractError::ExpectingLPToken(fund.to_string()));
    }

    let lp_amount = fund.amount;
    let sender = info.sender;

    state.assert_not_nested(deps.storage)?;

    let total_lp_supply = lp_token.total_supply;
    let assets = lsds.get_total_assets_err(deps.as_ref(), &env, &state, &config)?;
    let withdraw_amount = assets.vault_total.multiply_ratio(lp_amount, total_lp_supply);

    let mut response = if let Some(true) = immediate {
        // use full fee, zero unlocked
        create_withdraw_msgs(
            &deps.querier,
            deps.storage,
            &env,
            &state,
            &config,
            sender,
            withdraw_amount,
            Decimal::one(),
            Uint128::zero(),
        )?
    } else {
        let fee_config = state.fee_config.load(deps.storage)?;

        state.add_to_unbond_history(
            deps.storage,
            sender.clone(),
            UnbondHistory {
                amount_asset: withdraw_amount,
                start_time: env.block.time.seconds(),
                release_time: env.block.time.seconds() + config.unbond_time_s,
            },
        )?;

        let withdraw_protocol_fee = withdraw_amount * fee_config.protocol_withdraw_fee;
        let receive_amount = withdraw_amount.checked_sub(withdraw_protocol_fee)?;

        Response::new().add_attributes(vec![
            attr("action", "arb/execute_unbond"),
            attr("from", sender),
            attr("withdraw_amount", withdraw_amount),
            attr("receive_amount", receive_amount),
            attr("protocol_fee", withdraw_protocol_fee),
            attr("vault_total", assets.vault_total),
            attr("total_supply", total_lp_supply),
            attr("unbond_time_s", config.unbond_time_s.to_string()),
        ])
    };

    // always burn when receiving LP token
    response = response
        .add_message(create_burn_msg(&env, deps.storage, &state, &mut lp_token, lp_amount)?)
        .add_attribute("burnt_amount", lp_amount);

    Ok(response)
}

pub fn execute_withdraw_unbonding_immediate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    state.assert_not_nested(deps.storage)?;

    let key = (info.sender.clone(), id);
    let unbond_history = state.unbond_history.load(deps.storage, key.clone())?;

    let withdraw_amount = unbond_history.amount_asset;

    let withdraw_pool_fee_factor = unbond_history.pool_fee_factor(env.block.time.seconds());
    let response = create_withdraw_msgs(
        &deps.querier,
        deps.storage,
        &env,
        &state,
        &config,
        info.sender,
        withdraw_amount,
        withdraw_pool_fee_factor,
        withdraw_amount,
    )?;

    state.unbond_history.remove(deps.storage, key);

    Ok(response)
}

pub fn execute_withdraw_unbonded(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResult {
    let state = State::default();
    let config = state.config.load(deps.storage)?;
    state.assert_not_nested(deps.storage)?;

    let current_time = env.block.time.seconds();

    let unbond_history = state
        .unbond_history
        .prefix(info.sender.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .filter_ok(|element| element.1.release_time <= current_time)
        .take(30)
        .collect::<StdResult<Vec<(u64, UnbondHistory)>>>()?;

    // check that something can be withdrawn
    let withdraw_amount: Uint128 =
        unbond_history.iter().map(|element| element.1.amount_asset).sum();

    let response = create_withdraw_msgs(
        &deps.querier,
        deps.storage,
        &env,
        &state,
        &config,
        info.sender.clone(),
        withdraw_amount,
        Decimal::zero(),
        withdraw_amount,
    )?;

    // remove elements
    for (id, _) in unbond_history {
        state.unbond_history.remove(deps.storage, (info.sender.clone(), id));
    }

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn create_withdraw_msgs(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    env: &Env,
    state: &State,
    config: &ValidatedConfig,
    receiver: Addr,
    withdraw_amount: Uint128,
    withdraw_pool_fee_factor: Decimal,
    take_from_locked: Uint128,
) -> ContractResult {
    if withdraw_amount.is_zero() {
        return Err(ContractError::NoWithdrawableAsset {});
    }

    // check that enough assets are in the pool
    let balance_locked = state.balance_locked.load(storage)?;
    let locked_after = balance_locked.balance.checked_sub(take_from_locked).unwrap_or_default();
    let available_amount = config.query_utoken_amount(querier, env)?;

    // can only take immediate from not locked amount
    let takeable = available_amount.checked_sub(locked_after).unwrap_or_default();

    if takeable < withdraw_amount {
        return Err(ContractError::NotEnoughAssetsInThePool {});
    }

    state.balance_locked.save(
        storage,
        &BalanceLocked {
            balance: locked_after,
        },
    )?;

    let fee_config = state.fee_config.load(storage)?;

    let (withdraw_protocol_fee, withdraw_pool_fee) =
        calc_fees(&fee_config, withdraw_amount, withdraw_pool_fee_factor)?;

    let receive_amount =
        withdraw_amount.checked_sub(withdraw_protocol_fee)?.checked_sub(withdraw_pool_fee)?;

    let protocol_fee_msg = if !withdraw_protocol_fee.is_zero() {
        Some(
            native_asset(config.utoken.clone(), withdraw_protocol_fee)
                .into_msg(fee_config.protocol_fee_contract)?
                .to_specific()?,
        )
    } else {
        None
    };

    let withdraw_msg = native_asset(config.utoken.clone(), receive_amount)
        .into_msg(receiver.clone())?
        .to_specific()?;

    Ok(Response::<CustomMsgType>::new()
        // send assets to the sender
        .add_message(withdraw_msg)
        // send protocol fee
        .add_optional_message(protocol_fee_msg)
        .add_attributes(vec![
            attr("action", "arb/execute_withdraw"),
            attr("from", env.contract.address.clone()),
            attr("receiver", receiver),
            attr("withdraw_amount", withdraw_amount),
            attr("receive_amount", receive_amount),
            attr("protocol_fee", withdraw_protocol_fee),
            attr("pool_fee", withdraw_pool_fee),
            attr("immediate", (!withdraw_pool_fee.is_zero()).to_string()),
        ]))
}

fn create_burn_msg(
    env: &Env,
    storage: &mut dyn Storage,
    state: &State,
    lp_token: &mut LpToken,
    amount: Uint128,
) -> Result<CosmosMsg<CustomMsgType>, ContractError> {
    lp_token.total_supply = lp_token.total_supply.checked_sub(amount)?;
    state.lp_token.save(storage, lp_token)?;
    Ok(chain(env).create_burn_msg(lp_token.denom.clone(), amount))
}

fn create_mint_msgs(
    env: &Env,
    storage: &mut dyn Storage,
    state: &State,
    lp_token: &mut LpToken,
    recipient: Addr,
    amount: Uint128,
) -> Result<Vec<CosmosMsg<CustomMsgType>>, ContractError> {
    lp_token.total_supply = lp_token.total_supply.checked_add(amount)?;
    state.lp_token.save(storage, lp_token)?;
    Ok(chain(env).create_mint_msgs(lp_token.denom.clone(), amount, recipient))
}
