use crate::helpers::bps::BasicPoints;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cosmwasm_std::{StdError, StdResult};
use std::collections::HashMap;

/// An enum representing staking hooks.
#[cw_serde]
pub enum StakeChangedHookMsg {
    Stake {
        addr: Addr,
        amount: Uint128,
    },
    Unstake {
        addr: Addr,
        amount: Uint128,
    },
}

/// This structure describes the basic settings for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub report_debounce_s: u64,
    pub hook_sender_addr: String,
    pub restaking_hub_addr: String,
    pub min_gauge_percentage: Decimal,
}

/// This structure describes the execute messages available in the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Vote allows a vAMP holder to cast votes on which validators should get the delegations
    Vote {
        votes: Vec<(String, u16)>,
    },

    StakeChangeHook(StakeChangedHookMsg),

    UpdateConfig(UpdateConfigMsg),

    UpdateRestakeHub {},

    // Admin action to remove a user
    RemoveUser {
        user: String,
    },

    /// ProposeNewOwner proposes a new owner for the contract
    ProposeNewOwner {
        /// Newly proposed contract owner
        new_owner: String,
        /// The timestamp when the contract ownership change expires
        expires_in: u64,
    },
    /// DropOwnershipProposal removes the latest contract ownership transfer proposal
    DropOwnershipProposal {},
    /// ClaimOwnership allows the newly proposed owner to claim contract ownership
    ClaimOwnership {},
}
#[cw_serde]
pub struct UpdateConfigMsg {
    pub report_debounce_s: Option<u64>,
    pub hook_sender: Option<String>,
    pub min_gauge_percentage: Option<Decimal>,
    pub restake_hub_addr: Option<String>,
}

/// This structure describes the query messages available in the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// UserInfo returns information about a voter and the validators they voted for
    #[returns(UserInfoResponse)]
    UserInfo {
        user: String,
    },
    #[returns(UserInfosResponse)]
    UserInfos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Config returns the contract configuration
    #[returns(ConfigResponse)]
    Config {},
    /// PoolInfo returns the latest voting power allocated to a specific pool (generator)
    #[returns(VotedValidatorInfoResponse)]
    State {},
}

/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[cw_serde]
pub struct MigrateMsg {}

/// This structure describes the parameters returned when querying for the contract configuration.
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
    pub allowed_lps: Vec<String>,
}

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub report_debounce_s: u64,
    pub hook_sender_addr: Addr,
    pub restake_hub_addr: Addr,
    pub min_gauge_percentage: Decimal,
}

/// This structure describes voting parameters for a specific validator.
#[cw_serde]
pub struct StateResponse {
    pub global_votes: HashMap<String, Uint128>,
    pub update_time_s: u64,
    pub report_time_s: u64,
}

impl Config {
    pub fn assert_owner(&self, addr: &Addr) -> StdResult<()> {
        if *addr != self.owner {
            return Err(StdError::generic_err("unauthorized"));
        }
        Ok(())
    }

    pub fn assert_owner_or_self(&self, addr: &Addr, contract_addr: &Addr) -> StdResult<()> {
        if *addr != self.owner && *addr != *contract_addr {
            return Err(StdError::generic_err("unauthorized"));
        }
        Ok(())
    }
}
/// This structure describes the response used to return voting information for a specific pool (generator).
#[cw_serde]
#[derive(Default)]
pub struct VotedValidatorInfoResponse {
    /// Dynamic voting power that voted for this validator
    pub voting_power: Uint128,
    /// fixed amount available
    pub fixed_amount: Uint128,
    /// The slope at which the amount of vAMP that voted for this validator will decay
    pub slope: Uint128,
}

/// The struct describes a response used to return a staker's vAMP lock position.
#[cw_serde]
#[derive(Default)]
pub struct UserInfoResponse {
    pub vote_ts: u64,
    pub voting_power: Uint128,
    pub votes: Vec<(String, BasicPoints)>,
}

#[cw_serde]
#[derive(Default)]
pub struct UserInfosResponse {
    pub users: Vec<(Addr, UserInfoResponse)>,
}
