use astroport::common::OwnershipProposal;
use cosmwasm_schema::{
    cw_serde,
    schemars::JsonSchema,
    serde::{Deserialize, Serialize},
};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, SnapshotMap, Strategy};

/// This structure stores the main parameters for the voting escrow contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// Address that's allowed to change contract parameters
    pub owner: Addr,
    /// Address that can only blacklist vAMP stakers and remove their governance power
    pub guardian_addr: Option<Addr>,
    /// The ampLP token contract address
    pub deposit_denom: String,
    /// The list of whitelisted logo urls prefixes
    pub logo_urls_whitelist: Vec<String>,
    /// The list of contracts to receive updates on user's lock info changes
    pub push_update_contracts: Vec<Addr>,
    /// Address that can only blacklist vAMP stakers and remove their governance power
    #[serde(default)]
    pub decommissioned: Option<bool>,
}

/// This structure stores points along the checkpoint history for every vAMP staker.
#[cw_serde]
pub struct Point {
    /// The staker's vAMP voting power
    pub power: Uint128,
    /// The start period when the staker's voting power start to decrease
    pub start: u64,
    /// The period when the lock should expire
    pub end: u64,
    /// Weekly voting power decay
    pub slope: Uint128,

    /// fixed amount
    pub fixed: Uint128,
}

/// This structure stores data about the lockup position for a specific vAMP staker.
#[cw_serde]
pub struct Lock {
    /// The total amount of ampLP tokens that were deposited in the vAMP position
    pub amount: Uint128,
    /// The start period when the lock was created
    pub start: u64,
    /// The period when the lock position expires
    pub end: u64,
    /// the last period when the lock's time was increased
    pub last_extend_lock_period: u64,
}

/// Stores the contract config at the given key
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores all user lock history
pub const LOCKED: SnapshotMap<Addr, Lock> =
    SnapshotMap::new("locked", "locked__checkpoints", "locked__changelog", Strategy::EveryBlock);

/// Stores the checkpoint history for every staker (addr => period)
/// Total voting power checkpoints are stored using a (contract_addr => period) key
pub const HISTORY: Map<(Addr, u64), Point> = Map::new("history");

/// Scheduled slope changes per period (week)
pub const SLOPE_CHANGES: Map<u64, Uint128> = Map::new("slope_changes");

/// Last period when a scheduled slope change was applied
pub const LAST_SLOPE_CHANGE: Item<u64> = Item::new("last_slope_change");

/// Contains a proposal to change contract ownership
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");

/// Contains blacklisted staker addresses
pub const BLACKLIST: Item<Vec<Addr>> = Item::new("blacklist");
