use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};

/// This structure is used to return the lock information for a vAMP position.
#[cw_serde]
pub struct LockInfoResponse {
    /// The amount of ampLP locked in the position
    pub amount: Uint128,
    /// This is the initial boost for the lock position
    pub coefficient: Decimal,
    /// Start time for the vAMP position decay
    pub start: u64,
    /// End time for the vAMP position decay
    pub end: u64,
    /// Slope at which a staker's vAMP balance decreases over time
    pub slope: Uint128,

    /// fixed sockel
    pub fixed_amount: Uint128,
    /// includes only decreasing voting_power, it is the current voting power of the period currently queried.
    pub voting_power: Uint128,
}
