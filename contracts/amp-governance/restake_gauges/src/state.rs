use astroport::common::OwnershipProposal;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use eris::restake_gauges::{Config, StateResponse, UserInfoResponse};

pub type VoteState = StateResponse;
pub type UserInfo = UserInfoResponse;

/// Stores config at the given key.
pub const CONFIG: Item<Config> = Item::new("config");

/// State of the currently vote distribution
pub const VOTE_STATE: Item<VoteState> = Item::new("state");

/// User's voting information.
pub const USER_INFO: Map<&Addr, UserInfo> = Map::new("user_info");

/// Contains a proposal to change contract ownership
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");
