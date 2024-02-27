use cosmwasm_std::{Addr, Coin, Decimal, QuerierWrapper, StdError, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

use eris::{
    alliance_lst::AllianceStakeToken,
    hub::{
        Batch, DelegationStrategy, FeeConfig, PendingBatch, SingleSwapConfig, UnbondRequest,
        WantedDelegationsShare,
    },
};
use eris_chain_adapter::types::{CustomQueryType, DenomType, WithdrawType};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::ContractError,
    types::{alliance_delegations::AllianceDelegations, BooleanKey},
};

pub struct State<'a> {
    /// Account who can call certain privileged functions
    pub owner: Item<'a, Addr>,
    /// Account who can call harvest
    pub operator: Item<'a, Addr>,
    /// Stages that must be used by permissionless users
    pub stages_preset: Item<'a, Vec<Vec<SingleSwapConfig>>>,
    /// Withdraws that must be used by permissionless users
    pub withdrawals_preset: Item<'a, Vec<(WithdrawType, DenomType)>>,

    /// Pending ownership transfer, awaiting acceptance by the new owner
    pub new_owner: Item<'a, Addr>,
    /// Denom and supply of the Liquid Staking token
    pub stake_token: Item<'a, AllianceStakeToken>,
    /// How often the unbonding queue is to be executed
    pub epoch_period: Item<'a, u64>,
    /// The staking module's unbonding time, in seconds
    pub unbond_period: Item<'a, u64>,
    /// Validators who will receive the delegations
    pub validator_proxy: Item<'a, Addr>,
    pub validators_proxy_item: Item<'a, Vec<String>>,

    /// stores all delegations
    pub alliance_delegations: Item<'a, AllianceDelegations>,

    /// Coins that can be reinvested
    pub unlocked_coins: Item<'a, Vec<Coin>>,
    /// The current batch of unbonding requests queded to be executed
    pub pending_batch: Item<'a, PendingBatch>,
    /// Previous batches that have started unbonding but not yet finished
    pub previous_batches: IndexedMap<'a, u64, Batch, PreviousBatchesIndexes<'a>>,
    /// Users' shares in unbonding batches
    pub unbond_requests: IndexedMap<'a, (u64, &'a Addr), UnbondRequest, UnbondRequestsIndexes<'a>>,
    /// Fee Config
    pub fee_config: Item<'a, FeeConfig>,
    /// Delegation Strategy
    pub delegation_strategy: Item<'a, DelegationStrategy<Addr>>,
    /// Delegation Distribution
    pub delegation_goal: Item<'a, WantedDelegationsShare>,
    /// Specifies wether the contract allows donations
    pub allow_donations: Item<'a, bool>,

    // history of the exchange_rate
    pub exchange_history: Map<'a, u64, Decimal>,

    pub default_max_spread: Item<'a, u64>,
}

impl Default for State<'static> {
    fn default() -> Self {
        let pb_indexes = PreviousBatchesIndexes {
            reconciled: MultiIndex::new(
                |_, d: &Batch| d.reconciled.into(),
                "previous_batches",
                "previous_batches__reconciled",
            ),
        };
        let ubr_indexes = UnbondRequestsIndexes {
            user: MultiIndex::new(
                |_, d: &UnbondRequest| d.user.clone().into(),
                "unbond_requests",
                "unbond_requests__user",
            ),
        };
        Self {
            owner: Item::new("owner"),
            new_owner: Item::new("new_owner"),
            operator: Item::new("operator"),
            stages_preset: Item::new("stages_preset"),
            withdrawals_preset: Item::new("withdrawals_preset"),
            stake_token: Item::new("stake_token"),
            epoch_period: Item::new("epoch_period"),
            unbond_period: Item::new("unbond_period"),
            validator_proxy: Item::new("validator_proxy"),
            validators_proxy_item: Item::new("validators"),
            alliance_delegations: Item::new("alliance_delegations"),

            unlocked_coins: Item::new("unlocked_coins"),
            pending_batch: Item::new("pending_batch"),
            previous_batches: IndexedMap::new("previous_batches", pb_indexes),
            unbond_requests: IndexedMap::new("unbond_requests", ubr_indexes),
            fee_config: Item::new("fee_config"),
            delegation_strategy: Item::new("delegation_strategy"),
            delegation_goal: Item::new("delegation_goal"),
            allow_donations: Item::new("allow_donations"),
            exchange_history: Map::new("exchange_history"),
            default_max_spread: Item::new("default_max_spread"),
        }
    }
}

impl<'a> State<'a> {
    pub fn assert_owner(&self, storage: &dyn Storage, sender: &Addr) -> Result<(), ContractError> {
        let owner = self.owner.load(storage)?;
        if *sender == owner {
            Ok(())
        } else {
            Err(ContractError::Unauthorized {})
        }
    }

    pub fn assert_operator(
        &self,
        storage: &dyn Storage,
        sender: &Addr,
    ) -> Result<(), ContractError> {
        let operator = self.operator.load(storage)?;
        if *sender == operator {
            Ok(())
        } else {
            Err(ContractError::UnauthorizedSenderNotOperator {})
        }
    }

    pub fn get_or_preset<T>(
        &self,
        storage: &dyn Storage,
        stages: Option<Vec<T>>,
        preset: &Item<'static, Vec<T>>,
        sender: &Addr,
    ) -> Result<Option<Vec<T>>, ContractError>
    where
        T: Serialize + DeserializeOwned,
    {
        let stages = if let Some(stages) = stages {
            if stages.is_empty() {
                None
            } else {
                // only operator is allowed to send custom stages. Otherwise the contract would be able to interact with "bad contracts"
                // to fully decentralize, it would be required, that there is a whitelist of withdraw and swap contracts in the contract or somewhere else
                self.assert_operator(storage, sender)?;
                Some(stages)
            }
        } else {
            // otherwise use configured stages
            preset.may_load(storage)?
        };
        Ok(stages)
    }

    pub fn get_default_max_spread(&self, storage: &dyn Storage) -> Decimal {
        // by default a max_spread of 10% is used.
        Decimal::percent(self.default_max_spread.load(storage).unwrap_or(10))
    }

    pub fn get_validators(
        &self,
        storage: &dyn Storage,
        querier: &QuerierWrapper<CustomQueryType>,
    ) -> Result<Vec<String>, StdError> {
        let validator_proxy = self.validator_proxy.load(storage)?;
        self.validators_proxy_item.query(querier, validator_proxy)
    }
}

pub struct PreviousBatchesIndexes<'a> {
    // pk goes to second tuple element
    pub reconciled: MultiIndex<'a, BooleanKey, Batch, Vec<u8>>,
}

impl<'a> IndexList<Batch> for PreviousBatchesIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Batch>> + '_> {
        let v: Vec<&dyn Index<Batch>> = vec![&self.reconciled];
        Box::new(v.into_iter())
    }
}

pub struct UnbondRequestsIndexes<'a> {
    // pk goes to second tuple element
    pub user: MultiIndex<'a, String, UnbondRequest, (u64, &'a Addr)>,
}

impl<'a> IndexList<UnbondRequest> for UnbondRequestsIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<UnbondRequest>> + '_> {
        let v: Vec<&dyn Index<UnbondRequest>> = vec![&self.user];

        Box::new(v.into_iter())
    }
}
