use cosmwasm_std::{Coin, QuerierWrapper, StdResult};
use eris_chain_shared::alliance_query::{AllianceQuery, AllianceQueryWrapper};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DelegationResponse {
    pub denom: String,
    pub delegator: String,
    pub validator: String,
    pub amount: Coin,
}

pub struct AllianceQuerier<'a> {
    querier: &'a QuerierWrapper<'a, AllianceQueryWrapper>,
}

impl<'a> AllianceQuerier<'a> {
    pub fn new(querier: &'a QuerierWrapper<AllianceQueryWrapper>) -> Self {
        AllianceQuerier {
            querier,
        }
    }

    pub fn query_delegation(
        &self,
        denom: String,
        delegator: String,
        validator: String,
    ) -> StdResult<DelegationResponse> {
        let request = AllianceQuery::Delegation {
            denom,
            delegator,
            validator,
        }
        .into();

        self.querier.query(&request)
    }
}
