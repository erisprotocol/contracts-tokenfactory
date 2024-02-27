use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage, Uint128};
use itertools::Itertools;

use crate::state::State;

use super::{Delegation, Redelegation, Undelegation};

#[cw_serde]
pub struct AllianceDelegations {
    pub delegations: HashMap<String, Uint128>,
}

impl AllianceDelegations {
    pub fn query_all_delegations(&self, denom: &str) -> Vec<Delegation> {
        self.delegations
            .iter()
            .map(|(key, val)| Delegation {
                validator: key.to_string(),
                amount: val.u128(),
                denom: denom.to_string(),
            })
            .collect_vec()
    }

    pub fn query_delegation(&self, validator: &str, denom: &str) -> Delegation {
        match self.delegations.get(validator) {
            Some(data) => Delegation {
                validator: validator.to_string(),
                amount: data.u128(),
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
    ) -> StdResult<AllianceDelegations> {
        for undelegation in undelegations {
            let new_value = self
                .delegations
                .get(&undelegation.validator)
                .copied()
                .unwrap_or_default()
                .checked_sub(Uint128::new(undelegation.amount))?;

            self.delegations.insert(undelegation.validator.clone(), new_value);
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

            self.delegations.insert(redelegation.src.clone(), new_value_src);

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
