use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use eris::hub::{CallbackMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use eris_chain_adapter::types::main_denom;

use crate::constants::{CONTRACT_NAME, CONTRACT_VERSION};
use crate::error::{ContractError, ContractResult};
use crate::helpers::parse_received_fund;
use crate::state::State;
use crate::{execute, gov, queries};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult {
    execute::instantiate(deps, env, msg)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
    let api = deps.api;
    match msg {
        ExecuteMsg::Bond {
            receiver,
        } => execute::bond(
            deps,
            env,
            receiver.map(|s| api.addr_validate(&s)).transpose()?.unwrap_or(info.sender),
            parse_received_fund(&info.funds, main_denom())?,
            false,
        ),
        ExecuteMsg::Donate {} => execute::bond(
            deps,
            env,
            info.sender,
            parse_received_fund(&info.funds, main_denom())?,
            true,
        ),
        ExecuteMsg::WithdrawUnbonded {
            receiver,
        } => execute::withdraw_unbonded(
            deps,
            env,
            info.sender.clone(),
            receiver.map(|s| api.addr_validate(&s)).transpose()?.unwrap_or(info.sender),
        ),
        ExecuteMsg::AddValidator {
            validator,
        } => execute::add_validator(deps, info.sender, validator),
        ExecuteMsg::RemoveValidator {
            validator,
        } => execute::remove_validator(deps, env, info.sender, validator),
        ExecuteMsg::TransferOwnership {
            new_owner,
        } => execute::transfer_ownership(deps, info.sender, new_owner),
        ExecuteMsg::DropOwnershipProposal {} => execute::drop_ownership_proposal(deps, info.sender),
        ExecuteMsg::AcceptOwnership {} => execute::accept_ownership(deps, info.sender),
        ExecuteMsg::Harvest {
            withdrawals,
            stages,
        } => execute::harvest(deps, env, withdrawals, stages, info.sender),
        ExecuteMsg::TuneDelegations {} => execute::tune_delegations(deps, env, info.sender),
        ExecuteMsg::Rebalance {
            min_redelegation,
        } => execute::rebalance(deps, env, info.sender, min_redelegation),
        ExecuteMsg::Reconcile {} => execute::reconcile(deps, env),
        ExecuteMsg::SubmitBatch {} => execute::submit_batch(deps, env),
        ExecuteMsg::Vote {
            proposal_id,
            vote,
        } => gov::vote(deps, env, info, proposal_id, vote),
        ExecuteMsg::VoteWeighted {
            proposal_id,
            votes,
        } => gov::vote_weighted(deps, env, info, proposal_id, votes),
        ExecuteMsg::Callback(callback_msg) => callback(deps, env, info, callback_msg),

        ExecuteMsg::UpdateConfig {
            protocol_fee_contract,
            protocol_reward_fee,
            operator,
            stages_preset,
            allow_donations,
            delegation_strategy,
            vote_operator,
            chain_config,
            withdrawls_preset,
        } => execute::update_config(
            deps,
            info.sender,
            protocol_fee_contract,
            protocol_reward_fee,
            operator,
            stages_preset,
            withdrawls_preset,
            allow_donations,
            delegation_strategy,
            vote_operator,
            chain_config,
        ),
        ExecuteMsg::QueueUnbond {
            receiver,
        } => {
            let state = State::default();
            let stake_token = state.stake_token.load(deps.storage)?;

            if info.funds.len() != 1 {
                return Err(ContractError::ExpectingSingleCoin {});
            }

            if info.funds[0].denom != stake_token.denom {
                return Err(ContractError::ExpectingStakeToken(info.funds[0].denom.to_string()));
            }

            execute::queue_unbond(
                deps,
                env,
                api.addr_validate(&receiver.unwrap_or_else(|| info.sender.to_string()))?,
                info.funds[0].amount,
            )
        },
    }
}

fn callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback_msg: CallbackMsg,
) -> ContractResult {
    if env.contract.address != info.sender {
        return Err(ContractError::CallbackOnlyCalledByContract {});
    }

    match callback_msg {
        CallbackMsg::Reinvest {} => execute::reinvest(deps, env),
        CallbackMsg::WithdrawLps {
            withdrawals,
        } => execute::withdraw_lps(deps, env, withdrawals),
        CallbackMsg::SingleStageSwap {
            stage,
        } => execute::single_stage_swap(deps, env, stage),
        CallbackMsg::CheckReceivedCoin {
            snapshot,
            snapshot_stake,
        } => execute::callback_received_coins(deps, env, snapshot, snapshot_stake),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::config(deps)?),
        QueryMsg::State {} => to_binary(&queries::state(deps, env)?),
        QueryMsg::PendingBatch {} => to_binary(&queries::pending_batch(deps)?),
        QueryMsg::PreviousBatch(id) => to_binary(&queries::previous_batch(deps, id)?),
        QueryMsg::PreviousBatches {
            start_after,
            limit,
        } => to_binary(&queries::previous_batches(deps, start_after, limit)?),
        QueryMsg::UnbondRequestsByBatch {
            id,
            start_after,
            limit,
        } => to_binary(&queries::unbond_requests_by_batch(deps, id, start_after, limit)?),
        QueryMsg::UnbondRequestsByUser {
            user,
            start_after,
            limit,
        } => to_binary(&queries::unbond_requests_by_user(deps, user, start_after, limit)?),

        QueryMsg::UnbondRequestsByUserDetails {
            user,
            start_after,
            limit,
        } => to_binary(&queries::unbond_requests_by_user_details(
            deps,
            user,
            start_after,
            limit,
            env,
        )?),
        QueryMsg::WantedDelegations {} => to_binary(&queries::wanted_delegations(deps, env)?),
        QueryMsg::SimulateWantedDelegations {
            period,
        } => to_binary(&queries::simulate_wanted_delegations(deps, env, period)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult {
    // let contract_version = get_contract_version(deps.storage)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        // .add_attribute("previous_contract_name", &contract_version.contract)
        // .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
