use crate::error::ContractError;
use crate::state::{Config, State, CONFIG, STATE};
use astroport::asset::{token_asset, Asset};
use astroport::querier::query_token_balance;
use cosmwasm_std::{attr, Addr, Decimal, DepsMut, Env, MessageInfo, Response, Uint128};
use eris::adapters::asset::AssetEx;
use eris::adapters::token::Token;
use eris::astroport_farm::CallbackMsg;
use eris::helper::{assert_uniq_assets, funds_or_allowance};
use eris_chain_adapter::types::chain;
use eris_chain_shared::chain_trait::ChainInterface;

#[allow(clippy::too_many_arguments)]
/// ## Description
/// Send assets to compound proxy to create LP token and bond received LP token on behalf of sender.
pub fn bond_assets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: Vec<Asset>,
    minimum_receive: Option<Uint128>,
    no_swap: Option<bool>,
    slippage_tolerance: Option<Decimal>,
    receiver: Addr,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let lp_token = config.lp_token;

    assert_uniq_assets(&assets)?;

    let (funds, mut messages) =
        funds_or_allowance(&env, &config.compound_proxy.0, &assets, Some(&info))?;

    let compound = config.compound_proxy.compound_msg(
        assets,
        funds,
        no_swap,
        slippage_tolerance,
        &lp_token,
        None,
    )?;
    messages.push(compound);

    let prev_balance = query_token_balance(&deps.querier, lp_token, &env.contract.address)?;
    messages.push(
        CallbackMsg::BondTo {
            to: receiver,
            prev_balance,
            minimum_receive,
        }
        .into_cosmos_msg(&env.contract.address)?,
    );

    Ok(Response::new().add_messages(messages).add_attribute("action", "ampf/bond_assets"))
}

/// ## Description
/// Bond available LP token on the contract on behalf of the user.
pub fn bond_to(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    to: Addr,
    prev_balance: Uint128,
    minimum_receive: Option<Uint128>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let balance = query_token_balance(&deps.querier, &config.lp_token, &env.contract.address)?;
    let amount = balance - prev_balance;

    if let Some(minimum_receive) = minimum_receive {
        if amount < minimum_receive {
            return Err(ContractError::AssertionMinimumReceive {
                minimum_receive,
                amount,
            });
        }
    }

    bond_internal(deps, env, config, to, amount)
}

/// ## Description
/// Bond received LP token on behalf of the user.
pub fn bond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender_addr: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let staker_addr = deps.api.addr_validate(&sender_addr)?;

    let config = CONFIG.load(deps.storage)?;

    // only staking token contract can execute this message
    if config.lp_token != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    bond_internal(deps, env, config, staker_addr, amount)
}

/// Internal bond function used by bond and bond_to
fn bond_internal(
    deps: DepsMut,
    env: Env,
    config: Config,
    staker_addr: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let chain = chain(&env);
    let lp_balance = config.staking_contract.query_deposit(
        &deps.querier,
        &config.lp_token,
        &env.contract.address,
    )?;

    let mut state = STATE.load(deps.storage)?;

    // calculate share
    let bond_share = state.calc_bond_share(amount, lp_balance);
    let bond_share_adjusted =
        config.deposit_profit_delay.calc_adjusted_share(deps.storage, bond_share)?;

    state.total_bond_share += bond_share_adjusted;
    STATE.save(deps.storage, &state)?;

    let mut messages = match state.amp_lp_token {
        astroport::asset::AssetInfo::Token {
            contract_addr,
        } => vec![Token(contract_addr).mint(bond_share_adjusted, staker_addr)?],
        astroport::asset::AssetInfo::NativeToken {
            denom,
        } => chain.create_mint_msgs(denom, bond_share_adjusted, staker_addr),
    };

    messages.push(config.staking_contract.deposit_msg(config.lp_token.to_string(), amount)?);
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "ampf/bond"),
        attr("amount", amount),
        attr("bond_amount", amount),
        attr("bond_share_adjusted", bond_share_adjusted),
        attr("bond_share", bond_share),
    ]))
}

/// ## Description
/// Unbond LP token of sender
pub fn unbond(
    deps: DepsMut,
    env: Env,
    mut state: State,
    sender: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let chain = chain(&env);
    let staker_addr = deps.api.addr_validate(&sender)?;

    let config = CONFIG.load(deps.storage)?;
    let staking_token = config.lp_token;

    let lp_balance = config.staking_contract.query_deposit(
        &deps.querier,
        &staking_token,
        &env.contract.address,
    )?;

    let bond_amount = state.calc_bond_amount(lp_balance, amount);
    // update state
    state.total_bond_share = state.total_bond_share.checked_sub(amount)?;
    STATE.save(deps.storage, &state)?;

    let burn_msg = match state.amp_lp_token {
        astroport::asset::AssetInfo::Token {
            contract_addr,
        } => Token(contract_addr).burn(amount)?,
        astroport::asset::AssetInfo::NativeToken {
            denom,
        } => chain.create_burn_msg(denom, amount),
    };

    Ok(Response::new()
        .add_messages(vec![
            burn_msg,
            config.staking_contract.withdraw_msg(staking_token.to_string(), bond_amount)?,
            token_asset(staking_token, bond_amount).transfer_msg(&staker_addr)?,
        ])
        .add_attributes(vec![
            attr("action", "ampf/unbond"),
            attr("staker_addr", staker_addr),
            attr("amount", bond_amount),
        ]))
}
