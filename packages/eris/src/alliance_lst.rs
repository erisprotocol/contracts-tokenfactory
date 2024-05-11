use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};
use eris_chain_adapter::types::{DenomType, WithdrawType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::hub::{
    Batch, CallbackMsg, ClaimType, DelegationStrategy, DelegationsResponse, ExchangeRatesResponse,
    FeeConfig, PendingBatch, SingleSwapConfig, StateResponse, UnbondRequestsByBatchResponseItem,
    UnbondRequestsByUserResponseItem, UnbondRequestsByUserResponseItemDetails,
    WantedDelegationsResponse,
};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct AllianceStakeToken {
    // denom of the underlying token
    pub utoken: String,
    // denom of the stake token
    pub denom: String,
    // supply of the stake token
    pub total_supply: Uint128,
    // amount of utoken bonded
    pub total_utoken_bonded: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Account who can call certain privileged functions
    pub owner: String,

    /// Account who can call harvest
    pub operator: String,
    /// Denom of the underlaying staking token
    pub utoken: String,

    /// Name of the liquid staking token
    pub denom: String,
    /// How often the unbonding queue is to be executed, in seconds
    pub epoch_period: u64,
    /// The staking module's unbonding time, in seconds
    pub unbond_period: u64,
    /// Specifies a validators proxy contract, so that validators are not locally stored
    pub validator_proxy: String,

    /// Contract address where fees are sent
    pub protocol_fee_contract: String,
    /// Fees that are being applied during reinvest of staking rewards
    pub protocol_reward_fee: Decimal, // "1 is 100%, 0.05 is 5%"
    /// Strategy how delegations should be handled
    pub delegation_strategy: Option<DelegationStrategy>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Bond specified amount of Token
    Bond {
        receiver: Option<String>,
    },
    /// Donates specified amount of Token to pool
    Donate {},
    /// Withdraw Token that have finished unbonding in previous batches
    WithdrawUnbonded {
        receiver: Option<String>,
    },
    /// Transfer ownership to another account; will not take effect unless the new owner accepts
    TransferOwnership {
        new_owner: String,
    },
    /// Accept an ownership transfer
    AcceptOwnership {},
    /// Remove the ownership transfer proposal
    DropOwnershipProposal {},
    /// Claim staking rewards, swap all for Token, and restake
    Harvest {
        // specifies which validators should be harvested
        validators: Option<Vec<String>>,
        withdrawals: Option<Vec<(WithdrawType, DenomType)>>,
        stages: Option<Vec<Vec<SingleSwapConfig>>>,
    },

    TuneDelegations {},
    /// Use redelegations to balance the amounts of Token delegated to validators
    Rebalance {
        min_redelegation: Option<Uint128>,
    },
    /// Update Token amounts in unbonding batches to reflect any slashing or rounding errors
    Reconcile {},

    /// Reconciles if a slashing happened
    CheckSlashing {
        /// only when the current state equals the send amount, the slash will be applied
        state_total_utoken_bonded: Uint128,
        /// current delegations
        delegations: Vec<(String, Uint128)>,
    },
    /// Submit the current pending batch of unbonding requests to be unbonded
    SubmitBatch {
        undelegations: Option<Vec<Undelegation>>,
    },
    /// Callbacks; can only be invoked by the contract itself
    Callback(CallbackMsg),

    /// Updates the fee config,
    UpdateConfig {
        /// Contract address where fees are sent
        protocol_fee_contract: Option<String>,
        /// Fees that are being applied during reinvest of staking rewards
        protocol_reward_fee: Option<Decimal>, // "1 is 100%, 0.05 is 5%"
        /// Sets a new operator
        operator: Option<String>,
        /// Sets the stages preset
        stages_preset: Option<Vec<Vec<SingleSwapConfig>>>,
        /// Sets the withdrawals preset
        withdrawals_preset: Option<Vec<(WithdrawType, DenomType)>>,
        /// Specifies wether donations are allowed.
        allow_donations: Option<bool>,
        /// Strategy how delegations should be handled
        delegation_strategy: Option<DelegationStrategy>,
        /// Update the default max_spread
        default_max_spread: Option<u64>,

        /// How often the unbonding queue is to be executed, in seconds
        epoch_period: Option<u64>,
        /// The staking module's unbonding time, in seconds
        unbond_period: Option<u64>,
        /// Specifies a validators proxy contract, so that validators are not locally stored
        validator_proxy: Option<String>,
    },

    /// Submit an unbonding request to the current unbonding queue; automatically invokes `unbond`
    /// if `epoch_time` has elapsed since when the last unbonding queue was executed.
    QueueUnbond {
        receiver: Option<String>,
    },

    // Claim possible airdrops
    Claim {
        claims: Vec<ClaimType>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The contract's configurations. Response: `ConfigResponse`
    #[returns(ConfigResponse)]
    Config {},
    /// The contract's current state. Response: `StateResponse`
    #[returns(StateResponse)]
    State {},
    /// The contract's current delegation distribution goal. Response: `WantedDelegationsResponse`
    #[returns(WantedDelegationsResponse)]
    WantedDelegations {},
    /// The contract's delegation distribution goal based on period. Response: `WantedDelegationsResponse`
    #[returns(WantedDelegationsResponse)]
    SimulateWantedDelegations {
        /// by default uses the next period to look into the future.
        period: Option<u64>,
    },
    /// The current batch on unbonding requests pending submission. Response: `PendingBatch`
    #[returns(PendingBatch)]
    PendingBatch {},
    /// Query an individual batch that has previously been submitted for unbonding but have not yet
    /// fully withdrawn. Response: `Batch`
    #[returns(Batch)]
    PreviousBatch(u64),
    /// Enumerate all previous batches that have previously been submitted for unbonding but have not
    /// yet fully withdrawn. Response: `Vec<Batch>`
    #[returns(Vec<Batch>)]
    PreviousBatches {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Enumerate all outstanding unbonding requests in a given batch. Response: `Vec<UnbondRequestsByBatchResponseItem>`
    #[returns(Vec<UnbondRequestsByBatchResponseItem>)]
    UnbondRequestsByBatch {
        id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Enumreate all outstanding unbonding requests from given a user. Response: `Vec<UnbondRequestsByUserResponseItem>`
    #[returns(Vec<UnbondRequestsByUserResponseItem>)]
    UnbondRequestsByUser {
        user: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Enumreate all outstanding unbonding requests from given a user. Response: `Vec<UnbondRequestsByUserResponseItemDetails>`
    #[returns(Vec<UnbondRequestsByUserResponseItemDetails>)]
    UnbondRequestsByUserDetails {
        user: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(ExchangeRatesResponse)]
    ExchangeRates {
        // start after the provided timestamp in s
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(DelegationsResponse)]
    Delegations {},

    #[returns(Vec<Undelegation>)]
    SimulateUndelegations {},
}

#[cw_serde]
pub struct Undelegation {
    pub validator: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Account who can call certain privileged functions
    pub owner: String,
    /// Pending ownership transfer, awaiting acceptance by the new owner
    pub new_owner: Option<String>,
    /// Underlying staked token
    pub utoken: String,
    /// Address of the Stake token
    pub stake_token: String,

    /// How often the unbonding queue is to be executed, in seconds
    pub epoch_period: u64,
    /// The staking module's unbonding time, in seconds
    pub unbond_period: u64,
    /// Initial set of validators who will receive the delegations
    pub validators: Vec<String>,

    /// Information about applied fees
    pub fee_config: FeeConfig,

    /// Account who can call harvest
    pub operator: String,
    /// Stages that must be used by permissionless users
    pub stages_preset: Vec<Vec<SingleSwapConfig>>,
    /// withdrawals that must be used by permissionless users
    pub withdrawals_preset: Vec<(WithdrawType, DenomType)>,
    /// Specifies wether donations are allowed.
    pub allow_donations: bool,

    /// Strategy how delegations should be handled
    pub delegation_strategy: DelegationStrategy,

    pub validator_proxy: String,
}
