use std::{collections::HashSet, convert::TryInto};

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    to_binary, Addr, Api, Coin, CosmosMsg, Decimal, Empty, StdError, StdResult, Uint128,
    VoteOption, WasmMsg,
};
use eris_chain_adapter::types::{
    CustomMsgType, DenomType, HubChainConfigInput, StageType, WithdrawType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::helpers::bps::BasicPoints;

pub type SingleSwapConfig = (StageType, DenomType, Option<Decimal>);

#[cw_serde]
pub enum DelegationStrategy
//<T = String>
{
    /// all validators receive the same delegation.
    Uniform,
    Defined {
        shares_bps: Vec<(String, u16)>,
    },
    // /// validators receive delegations based on community voting + merit points
    // Gauges {
    //     /// gauges based on vAmp voting
    //     amp_gauges: T,
    //     /// gauges based on eris merit points
    //     emp_gauges: Option<T>,
    //     /// weight between amp and emp gauges between 0 and 1
    //     amp_factor_bps: u16,
    //     /// min amount of delegation needed
    //     min_delegation_bps: u16,
    //     /// max amount of delegation needed
    //     max_delegation_bps: u16,
    //     /// count of validators that should receive delegations
    //     validator_count: u8,
    // },
}

impl DelegationStrategy //<String>
{
    pub fn validate(
        self,
        _api: &dyn Api,
        validators: &[String],
    ) -> StdResult<
        DelegationStrategy, //<Addr>
    > {
        let result = match self {
            DelegationStrategy::Uniform {} => DelegationStrategy::Uniform {},

            // DelegationStrategy::Gauges {
            //     amp_gauges,
            //     emp_gauges,
            //     amp_factor_bps: amp_factor,
            //     min_delegation_bps,
            //     validator_count,
            //     max_delegation_bps,
            // } => DelegationStrategy::Gauges {
            //     amp_gauges: api.addr_validate(&amp_gauges)?,
            //     emp_gauges: addr_opt_validate(api, &emp_gauges)?,
            //     amp_factor_bps: amp_factor,
            //     min_delegation_bps,
            //     validator_count,
            //     max_delegation_bps,
            // },
            DelegationStrategy::Defined {
                shares_bps,
            } => {
                let mut duplicates = HashSet::new();
                let bps = shares_bps
                    .iter()
                    .map(|(validator, d)| {
                        if !validators.contains(validator) {
                            return Err(StdError::generic_err(format!(
                                "validator {0} not whitelisted",
                                validator
                            )))?;
                        }

                        if !duplicates.insert(validator.to_string()) {
                            return Err(StdError::generic_err(format!(
                                "validator {0} duplicated",
                                validator
                            )))?;
                        }

                        let bps: BasicPoints = (*d).try_into()?;
                        Ok(bps)
                    })
                    .collect::<StdResult<Vec<BasicPoints>>>()?;

                let result = bps
                    .iter()
                    .try_fold(BasicPoints::default(), |acc, bps| acc.checked_add(*bps))?;

                if !result.is_max() {
                    Err(StdError::generic_err("sum of shares is not 10000"))?;
                }

                DelegationStrategy::Defined {
                    shares_bps,
                }
            },
        };
        Ok(result)
    }
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
    /// Initial set of validators who will receive the delegations
    pub validators: Vec<String>,

    /// Contract address where fees are sent
    pub protocol_fee_contract: String,
    /// Fees that are being applied during reinvest of staking rewards
    pub protocol_reward_fee: Decimal, // "1 is 100%, 0.05 is 5%"
    /// Strategy how delegations should be handled
    pub delegation_strategy: Option<DelegationStrategy>,
    /// Contract address that is allowed to vote
    pub vote_operator: Option<String>,

    /// Chain specific config
    pub chain_config: HubChainConfigInput,
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
    /// Add a validator to the whitelist; callable by the owner
    AddValidator {
        validator: String,
    },
    /// Remove a validator from the whitelist; callable by the owner
    RemoveValidator {
        validator: String,
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
    /// Submit the current pending batch of unbonding requests to be unbonded
    SubmitBatch {},
    /// Vote on a proposal (only allowed by the vote_operator)
    Vote {
        proposal_id: u64,
        vote: VoteOption,
    },
    /// Vote on a proposal weighted (only allowed by the vote_operator)
    VoteWeighted {
        proposal_id: u64,
        votes: Vec<(Decimal, VoteOption)>,
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
        /// Update the vote_operator
        vote_operator: Option<String>,
        /// Update the chain_config
        chain_config: Option<HubChainConfigInput>,
        /// Update the default max_spread
        default_max_spread: Option<u64>,
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
pub enum CallbackMsg {
    WithdrawLps {
        withdrawals: Vec<(WithdrawType, DenomType)>,
    },
    // SingleStageSwap is executed multiple times to execute each swap stage. A stage consists of multiple swaps
    SingleStageSwap {
        // (Used dex, used denom, belief_price)
        stage: Vec<SingleSwapConfig>,
    },
    /// Following the swaps, stake the Token acquired to the whitelisted validators
    Reinvest {},

    CheckReceivedCoin {
        snapshot: Coin,
        snapshot_stake: Coin,
    },
}

impl CallbackMsg {
    pub fn into_cosmos_msg(&self, contract_addr: &Addr) -> StdResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ExecuteMsg::Callback(self.clone()))?,
            funds: vec![],
        }))
    }
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
    pub delegation_strategy: DelegationStrategy, //<String>,
    /// Update the vote_operator
    pub vote_operator: Option<String>,
}

#[cw_serde]
pub struct StateResponse {
    /// Total supply to the Stake token
    pub total_ustake: Uint128,
    /// Total amount of utoken staked (bonded)
    pub total_utoken: Uint128,
    /// The exchange rate between ustake and utoken, in terms of utoken per ustake
    pub exchange_rate: Decimal,
    /// Staking rewards currently held by the contract that are ready to be reinvested
    pub unlocked_coins: Vec<Coin>,
    // Amount of utoken currently unbonding
    pub unbonding: Uint128,
    // Amount of utoken currently available as balance of the contract
    pub available: Uint128,
    // Total amount of utoken within the contract (bonded + unbonding + available)
    pub tvl_utoken: Uint128,
}

#[cw_serde]
pub struct WantedDelegationsResponse {
    pub tune_time_period: Option<(u64, u64)>,
    pub delegations: Vec<(String, Uint128)>,
}

#[cw_serde]
pub struct WantedDelegationsShare {
    pub tune_time: u64,
    pub tune_period: u64,
    pub shares: Vec<(String, Decimal)>,
}

#[cw_serde]
pub struct PendingBatch {
    /// ID of this batch
    pub id: u64,
    /// Total amount of `ustake` to be burned in this batch
    pub ustake_to_burn: Uint128,
    /// Estimated time when this batch will be submitted for unbonding
    pub est_unbond_start_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct StakeToken {
    // denom of the underlying token
    pub utoken: String,
    // denom of the stake token
    pub denom: String,
    // supply of the stake token
    pub total_supply: Uint128,
}

#[cw_serde]
pub struct FeeConfig {
    /// Contract address where fees are sent
    pub protocol_fee_contract: Addr,
    /// Fees that are being applied during reinvest of staking rewards
    pub protocol_reward_fee: Decimal, // "1 is 100%, 0.05 is 5%"
}

#[cw_serde]
pub struct Batch {
    /// ID of this batch
    pub id: u64,
    /// Whether this batch has already been reconciled
    pub reconciled: bool,
    /// Total amount of shares remaining this batch. Each `ustake` burned = 1 share
    pub total_shares: Uint128,
    /// Amount of `utoken` in this batch that have not been claimed
    pub utoken_unclaimed: Uint128,
    /// Estimated time when this batch will finish unbonding
    pub est_unbond_end_time: u64,
}

#[cw_serde]
pub struct UnbondRequest {
    /// ID of the batch
    pub id: u64,
    /// The user's address
    pub user: Addr,
    /// The user's share in the batch
    pub shares: Uint128,
}

#[cw_serde]
pub struct UnbondRequestsByBatchResponseItem {
    /// The user's address
    pub user: String,
    /// The user's share in the batch
    pub shares: Uint128,
}

impl From<UnbondRequest> for UnbondRequestsByBatchResponseItem {
    fn from(s: UnbondRequest) -> Self {
        Self {
            user: s.user.into(),
            shares: s.shares,
        }
    }
}

#[cw_serde]
pub struct UnbondRequestsByUserResponseItem {
    /// ID of the batch
    pub id: u64,
    /// The user's share in the batch
    pub shares: Uint128,
}

impl From<UnbondRequest> for UnbondRequestsByUserResponseItem {
    fn from(s: UnbondRequest) -> Self {
        Self {
            id: s.id,
            shares: s.shares,
        }
    }
}

#[cw_serde]
pub struct UnbondRequestsByUserResponseItemDetails {
    /// ID of the batch
    pub id: u64,
    /// The user's share in the batch
    pub shares: Uint128,

    // state of pending, unbonding or completed
    pub state: String,

    // The details of the unbonding batch
    pub batch: Option<Batch>,

    // Is set if the unbonding request is still pending
    pub pending: Option<PendingBatch>,
}

#[cw_serde]
pub struct ExchangeRatesResponse {
    pub exchange_rates: Vec<(u64, Decimal)>,
    // APR normalized per DAY
    pub apr: Option<Decimal>,
}

#[cw_serde]
pub enum ClaimType {
    Default(String),
}

pub type MigrateMsg = Empty;
