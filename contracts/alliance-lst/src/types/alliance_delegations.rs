use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage, Uint128};
use eris::alliance_lst::Undelegation;
use itertools::Itertools;

use crate::{error::ContractError, state::State};

use super::{Delegation, Redelegation};

#[cw_serde]
pub struct AllianceDelegations {
    pub delegations: HashMap<String, Uint128>,
}

const REDUCTION_AMOUNT: u128 = 10u128;

impl AllianceDelegations {
    pub fn query_all_delegations(&self, denom: &str) -> Vec<Delegation> {
        self.delegations
            .iter()
            .map(|(key, amount)| Delegation {
                validator: key.to_string(),
                // due to rounding, the alliance module sometimes rounds down by 10
                // this can be fixed by applying slashings, but this would require running slash detection on many blocks
                // slash protection only triggeres when the delegations are more than 1 uunit differently.
                amount: amount.u128().saturating_sub(REDUCTION_AMOUNT),
                denom: denom.to_string(),
            })
            .collect_vec()
    }

    pub fn query_delegation(&self, validator: &str, denom: &str) -> Delegation {
        match self.delegations.get(validator) {
            Some(amount) => Delegation {
                validator: validator.to_string(),
                // due to rounding, the alliance module sometimes rounds down by 10
                // this can be fixed by applying slashings, but this would require running slash detection on many blocks
                // slash protection only triggeres when the delegations are more than 1 uunit differently.
                amount: amount.u128().saturating_sub(REDUCTION_AMOUNT),
                denom: denom.into(),
            },
            None => Delegation {
                validator: validator.to_string(),
                amount: 0,
                denom: "".into(),
            },
        }
    }

    pub fn delegate(mut self, delegation: &Delegation) -> StdResult<AllianceDelegations> {
        let new_value = self
            .delegations
            .get(&delegation.validator)
            .copied()
            .unwrap_or_default()
            .checked_add(Uint128::new(delegation.amount))?;

        self.delegations.insert(delegation.validator.clone(), new_value);
        Ok(self)
    }

    pub fn undelegate(
        mut self,
        undelegations: &Vec<Undelegation>,
    ) -> Result<AllianceDelegations, ContractError> {
        for undelegation in undelegations {
            let new_value = self
                .delegations
                .get(&undelegation.validator)
                .copied()
                .ok_or(ContractError::SubmitBatchFailure(format!(
                    "validator not found {0}",
                    undelegation.validator
                )))?
                .checked_sub(undelegation.amount)?;

            if new_value.is_zero() {
                self.delegations.remove(&undelegation.validator);
            } else {
                self.delegations.insert(undelegation.validator.clone(), new_value);
            }
        }

        Ok(self)
    }

    pub fn redelegate(
        mut self,
        redelegations: &Vec<Redelegation>,
    ) -> StdResult<AllianceDelegations> {
        for redelegation in redelegations {
            let new_value_src = self
                .delegations
                .get(&redelegation.src)
                .copied()
                .unwrap_or_default()
                .checked_sub(Uint128::new(redelegation.amount))?;

            if new_value_src.is_zero() {
                self.delegations.remove(&redelegation.src);
            } else {
                self.delegations.insert(redelegation.src.clone(), new_value_src);
            }

            let new_value_dst = self
                .delegations
                .get(&redelegation.dst)
                .copied()
                .unwrap_or_default()
                .checked_add(Uint128::new(redelegation.amount))?;

            self.delegations.insert(redelegation.dst.clone(), new_value_dst);
        }

        Ok(self)
    }

    pub fn save(self, state: &State, storage: &mut dyn Storage) -> StdResult<AllianceDelegations> {
        state.alliance_delegations.save(storage, &self)?;
        Ok(self)
    }
}
