use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::str::FromStr;

use crate::adapters::daodao::{DaoDao, VotingPowerAtHeightResponse};
use crate::adapters::restakehub::{AssetDistribution, RestakeHub};
use crate::error::ContractError;
use crate::state::{UserInfo, VoteState, CONFIG, OWNERSHIP_PROPOSAL, USER_INFO, VOTE_STATE};
use astroport::common::{claim_ownership, drop_ownership_proposal, propose_new_owner};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_storage_plus::Bound;
use eris::helpers::bps::BasicPoints;
use eris::restake_gauges::{
    Config, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StakeChangedHookMsg,
    UpdateConfigMsg, UserInfoDetailsResponse, UserInfoResponse, UserInfosResponse,
};
use eris::CustomResponse;
use itertools::Itertools;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "restake-gauges";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MAX_LIMIT: u32 = 100;
/// The default amount of items to read from
pub const DEFAULT_LIMIT: u32 = 10;

type ExecuteResult = Result<Response, ContractError>;

/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            report_debounce_s: msg.report_debounce_s,
            hook_sender_addr: deps.api.addr_validate(&msg.hook_sender_addr)?,
            min_gauge_percentage: assert_percentage(msg.min_gauge_percentage)?,
            restake_hub_addr: deps.api.addr_validate(&msg.restaking_hub_addr)?,
        },
    )?;

    VOTE_STATE.save(
        deps.storage,
        &VoteState {
            global_votes: HashMap::new(),
            update_time_s: 0,
            report_time_s: 0,
        },
    )?;

    Ok(Response::default())
}

fn assert_percentage(min_percentage: Decimal) -> Result<Decimal, ContractError> {
    if min_percentage > Decimal::from_ratio(20u128, 100u128) {
        Err(ContractError::ConfigError("Min percentage between 0-20%".to_string()))
    } else {
        Ok(min_percentage)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ExecuteResult {
    match msg {
        ExecuteMsg::Vote {
            votes,
        } => handle_vote(deps, env, info, votes),
        ExecuteMsg::StakeChangeHook(msg) => update_vote(deps, env, info, msg),
        ExecuteMsg::RemoveUser {
            user,
        } => remove_user(deps, env, info, user),
        ExecuteMsg::UpdateConfig(msg) => update_config(deps, info, msg),
        ExecuteMsg::UpdateRestakeHub {} => update_restake_hub(deps, env),
        ExecuteMsg::WhitelistAssets(assets) => {
            whitelist_assets_restake_hub(deps, env, info, assets)
        },
        ExecuteMsg::ProposeNewOwner {
            new_owner,
            expires_in,
        } => {
            let config: Config = CONFIG.load(deps.storage)?;

            propose_new_owner(
                deps,
                info,
                env,
                new_owner,
                expires_in,
                config.owner,
                OWNERSHIP_PROPOSAL,
            )
            .map_err(Into::into)
        },
        ExecuteMsg::DropOwnershipProposal {} => {
            let config: Config = CONFIG.load(deps.storage)?;

            drop_ownership_proposal(deps, info, config.owner, OWNERSHIP_PROPOSAL)
                .map_err(Into::into)
        },
        ExecuteMsg::ClaimOwnership {} => {
            claim_ownership(deps, info, env, OWNERSHIP_PROPOSAL, |deps, new_owner| {
                CONFIG
                    .update::<_, StdError>(deps.storage, |mut v| {
                        v.owner = new_owner;
                        Ok(v)
                    })
                    .map(|_| ())
            })
            .map_err(Into::into)
        },
    }
}

/// The function checks that:
/// * the user voting power is > 0,
/// * all pool addresses are valid LP token addresses,
/// * 'votes' vector doesn't contain duplicated pool addresses,
/// * sum of all BPS values <= 10000.
///
/// The function cancels changes applied by previous votes and apply new votes for the next period.
/// New vote parameters are saved in [`USER_INFO`].
///
/// The function returns [`Response`] in case of success or [`ContractError`] in case of errors.
///
/// * **votes** is a vector of pairs ([`String`], [`u16`]).
///     Tuple consists of pool address and percentage of user's voting power for a given pool.
///     Percentage should be in BPS form.
fn handle_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    votes: Vec<(String, u16)>,
) -> ExecuteResult {
    let user = info.sender;
    let config = CONFIG.load(deps.storage)?;

    let daodao = DaoDao(config.hook_sender_addr.clone());

    let voting_power = daodao.get_voting_power(&deps.querier, user.to_string())?.power;
    if voting_power.is_zero() {
        return Err(ContractError::ZeroVotingPower {});
    }

    let allowed_lps =
        RestakeHub(config.restake_hub_addr.clone()).get_whitelisted_assets(&deps.querier)?;

    let mut addrs_set = HashSet::new();
    let mut total = BasicPoints::default();
    // Validating addrs and bps
    let votes = votes
        .into_iter()
        .map(|(addr, bps)| {
            if !allowed_lps.contains(&addr) {
                return Err(ContractError::InvalidGaugeKey(addr));
            }
            // Check duplicated votes
            if !addrs_set.insert(addr.to_string()) {
                return Err(ContractError::DuplicatedGauges {});
            }

            let bps: BasicPoints = bps.try_into()?;
            // Check the bps sum is within the limit
            total = total.checked_add(bps)?;
            Ok((addr, bps))
        })
        .collect::<Result<Vec<_>, ContractError>>()?;

    let mut vote_state = VOTE_STATE.load(deps.storage)?;
    let mut user_info = USER_INFO.may_load(deps.storage, &user)?.unwrap_or_default();

    remove_votes_of_user(&mut vote_state, &user_info)?;

    user_info.votes = votes;
    user_info.voting_power = voting_power;

    apply_votes_of_user(&mut vote_state, &user_info)?;

    USER_INFO.save(deps.storage, &user, &user_info)?;

    Ok(Response::new()
        .add_attribute("action", "erisrestake/vote")
        .add_optional_message(save_and_update_restake_hub_msg(
            deps,
            env,
            &config,
            vote_state,
            Some(allowed_lps),
            false,
        )?)
        .add_attribute("voting_power", voting_power))
}

fn apply_votes_of_user(
    vote_state: &mut VoteState,
    user_info: &UserInfo,
) -> Result<(), ContractError> {
    if user_info.voting_power.is_zero() {
        return Ok(());
    }

    for (key, bps) in user_info.votes.iter() {
        let to_add = *bps * user_info.voting_power;
        let key_string = key.to_string();

        if let Some(amount) = vote_state.global_votes.get_mut(&key_string) {
            *amount = amount.checked_add(to_add)?;
        } else {
            vote_state.global_votes.insert(key_string, to_add);
        }
    }

    Ok(())
}

fn remove_votes_of_user(
    vote_state: &mut VoteState,
    user_info: &UserInfo,
) -> Result<(), ContractError> {
    for (key, bps) in user_info.votes.iter() {
        let to_remove = *bps * user_info.voting_power;
        let key_string = key.to_string();

        if let Some(amount) = vote_state.global_votes.get_mut(&key_string) {
            *amount = amount.saturating_sub(to_remove);

            if amount.is_zero() {
                vote_state.global_votes.remove(&key_string);
            }
        }
    }

    Ok(())
}

fn update_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    hook: StakeChangedHookMsg,
) -> ExecuteResult {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.hook_sender_addr {
        return Err(ContractError::Unauthorized {});
    }

    let (user, added, removed) = match hook {
        StakeChangedHookMsg::Stake {
            addr,
            amount,
        } => (addr, amount, Uint128::zero()),
        StakeChangedHookMsg::Unstake {
            addr,
            amount,
        } => (addr, Uint128::zero(), amount),
    };

    let user_info = USER_INFO.may_load(deps.storage, &user)?;

    if let Some(mut user_info) = user_info {
        let mut vote_state = VOTE_STATE.load(deps.storage)?;

        remove_votes_of_user(&mut vote_state, &user_info)?;

        user_info.voting_power = user_info.voting_power.checked_add(added)?.saturating_sub(removed);
        if user_info.voting_power.is_zero() {
            USER_INFO.remove(deps.storage, &user);
            VOTE_STATE.save(deps.storage, &vote_state)?;
            return Ok(Response::new().add_attribute("action", "erisrestake/update_vote_removed"));
        }
        apply_votes_of_user(&mut vote_state, &user_info)?;

        USER_INFO.save(deps.storage, &user, &user_info)?;

        return Ok(Response::new()
            .add_attribute("action", "erisrestake/update_vote_changed")
            .add_optional_message(save_and_update_restake_hub_msg(
                deps, env, &config, vote_state, None, false,
            )?)
            .add_attribute("voting_power", user_info.voting_power));
    }

    Ok(Response::new().add_attribute("action", "erisrestake/update_vote_noop"))
}

fn remove_user(deps: DepsMut, env: Env, info: MessageInfo, user: String) -> ExecuteResult {
    let config = CONFIG.load(deps.storage)?;
    config.assert_owner(&info.sender)?;

    let user = deps.api.addr_validate(&user)?;
    let user_info = USER_INFO.may_load(deps.storage, &user)?;

    if let Some(user_info) = user_info {
        USER_INFO.remove(deps.storage, &user);
        let mut vote_state = VOTE_STATE.load(deps.storage)?;

        remove_votes_of_user(&mut vote_state, &user_info)?;

        return Ok(Response::new()
            .add_optional_message(save_and_update_restake_hub_msg(
                deps, env, &config, vote_state, None, false,
            )?)
            .add_attribute("action", "erisrestake/remove_user"));
    }

    Ok(Response::new().add_attribute("action", "erisrestake/remove_user_noop"))
}

fn update_restake_hub(deps: DepsMut, env: Env) -> ExecuteResult {
    let config = CONFIG.load(deps.storage)?;
    let vote_state = VOTE_STATE.load(deps.storage)?;

    Ok(Response::new()
        .add_optional_message(save_and_update_restake_hub_msg(
            deps, env, &config, vote_state, None, true,
        )?)
        .add_attribute("action", "erisrestake/update_restake_hub"))
}

fn whitelist_assets_restake_hub(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    assets: HashMap<String, Vec<AssetInfo>>,
) -> ExecuteResult {
    let config = CONFIG.load(deps.storage)?;
    config.assert_owner(&info.sender)?;

    Ok(Response::new()
        .add_message(RestakeHub(config.restake_hub_addr).whitelist_assets_msg(assets)?)
        .add_attribute("action", "erisrestake/whitelist_assets_restake_hub"))
}

fn save_and_update_restake_hub_msg(
    deps: DepsMut,
    env: Env,
    config: &Config,
    mut vote_state: VoteState,
    allowed_lps: Option<HashSet<String>>,
    force_update: bool,
) -> StdResult<Option<CosmosMsg>> {
    let msg = if force_update
        || config.report_debounce_s == 0
        || env.block.time.seconds() > vote_state.report_time_s + config.report_debounce_s
    {
        let allowed_lps = if let Some(allowed_lps) = allowed_lps {
            allowed_lps
        } else {
            RestakeHub(config.restake_hub_addr.clone()).get_whitelisted_assets(&deps.querier)?
        };

        let allowed_votes = vote_state
            .global_votes
            .clone()
            .into_iter()
            .filter(|(lp, _)| allowed_lps.contains(lp))
            .collect_vec();

        let total_voting_power: Uint128 = allowed_votes.iter().map(|(_, amount)| amount).sum();
        let min_voting_power = config.min_gauge_percentage * total_voting_power;

        let relevant_votes = allowed_votes
            .into_iter()
            .filter(|(_, amount)| *amount > min_voting_power)
            .collect_vec();
        let sum_relevant: Uint128 = relevant_votes.iter().map(|(_, amount)| amount).sum();

        let mut distirbutions = relevant_votes
            .into_iter()
            .map(|(lp, amount)| {
                Ok(AssetDistribution {
                    asset: AssetInfoUnchecked::from_str(&lp)?.check(deps.api, None)?,
                    distribution: Decimal::from_ratio(amount, sum_relevant),
                })
            })
            .collect::<StdResult<Vec<_>>>()?;

        let total: Decimal = distirbutions.iter().map(|a| a.distribution).sum();

        if total > Decimal::percent(100) {
            let remove = total - Decimal::percent(100);
            distirbutions[0].distribution -= remove;
        } else {
            let add = Decimal::percent(100) - total;
            distirbutions[0].distribution += add;
        }

        vote_state.report_time_s = env.block.time.seconds();

        Some(RestakeHub(config.restake_hub_addr.clone()).set_asset_rewards_msg(distirbutions)?)
    } else {
        None
    };

    vote_state.update_time_s = env.block.time.seconds();

    VOTE_STATE.save(deps.storage, &vote_state)?;

    Ok(msg)
}

fn update_config(deps: DepsMut, info: MessageInfo, msg: UpdateConfigMsg) -> ExecuteResult {
    let mut config = CONFIG.load(deps.storage)?;

    config.assert_owner(&info.sender)?;

    if let Some(hook_sender) = msg.hook_sender {
        config.hook_sender_addr = deps.api.addr_validate(&hook_sender)?;
    }

    if let Some(min_gauge_percentage) = msg.min_gauge_percentage {
        config.min_gauge_percentage = assert_percentage(min_gauge_percentage)?;
    }

    if let Some(report_debounce_s) = msg.report_debounce_s {
        config.report_debounce_s = report_debounce_s;
    }

    if let Some(restake_hub_addr) = msg.restake_hub_addr {
        config.restake_hub_addr = deps.api.addr_validate(&restake_hub_addr)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "erisrestake/update_config"))
}

/// Expose available contract queries.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::UserInfo {
            user,
        } => to_json_binary(&user_info(deps, user)?),
        QueryMsg::UserInfos {
            start_after,
            limit,
        } => to_json_binary(&user_infos(deps, start_after, limit)?),
        QueryMsg::Config {} => to_json_binary(&config(deps)?),
        QueryMsg::State {} => to_json_binary(&VOTE_STATE.load(deps.storage)?),
    }
}

fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let allowed_lps =
        RestakeHub(config.restake_hub_addr.clone()).get_whitelisted_assets(&deps.querier)?;

    Ok(ConfigResponse {
        config,
        allowed_lps: allowed_lps.into_iter().collect_vec(),
    })
}

/// Returns user information.
fn user_info(deps: Deps, address: String) -> StdResult<UserInfoDetailsResponse> {
    let config = CONFIG.load(deps.storage)?;

    let user_addr = deps.api.addr_validate(&address)?;
    let user = USER_INFO.may_load(deps.storage, &user_addr)?;

    let daodao = DaoDao(config.hook_sender_addr.clone());
    let staked = daodao
        .get_voting_power(&deps.querier, address.to_string())
        .unwrap_or(VotingPowerAtHeightResponse {
            height: 0,
            power: Uint128::zero(),
        })
        .power;

    Ok(UserInfoDetailsResponse {
        user,
        staked,
    })
}

// returns all user votes
fn user_infos(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<UserInfosResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let mut start: Option<Bound<&Addr>> = None;
    let addr: Addr;
    if let Some(start_after) = start_after {
        if let Ok(start_after_addr) = deps.api.addr_validate(&start_after) {
            addr = start_after_addr;
            start = Some(Bound::exclusive(&addr));
        }
    }

    let users = USER_INFO
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<(Addr, UserInfoResponse)>>>()?;

    Ok(UserInfosResponse {
        users,
    })
}

/// Manages contract migration
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if contract_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err(format!(
            "contract_name does not match: prev: {0}, new: {1}",
            contract_version.contract, CONTRACT_VERSION
        ))
        .into());
    }

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}

#[test]
fn test_remove() {
    let mut global_votes = HashMap::new();
    global_votes.insert("cw:test".to_string(), Uint128::new(1000));
    global_votes.insert("native:test".to_string(), Uint128::new(10000));
    let mut state = VoteState {
        global_votes,
        update_time_s: 0,
        report_time_s: 0,
    };
    let user = UserInfoResponse {
        vote_ts: 0,
        votes: vec![("cw:test".to_string(), 5000u128.try_into().unwrap())],
        voting_power: Uint128::new(100),
    };
    remove_votes_of_user(&mut state, &user).unwrap();

    let mut global_votes = HashMap::new();
    global_votes.insert("cw:test".to_string(), Uint128::new(950));
    global_votes.insert("native:test".to_string(), Uint128::new(10000));
    let new_state = VoteState {
        global_votes,
        update_time_s: 0,
        report_time_s: 0,
    };
    assert_eq!(state, new_state)
}
