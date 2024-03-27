use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdResult};
use cosmwasm_std::{QuerierWrapper, Uint128};

#[cw_serde]
pub enum QueryMsg {
    VotingPowerAtHeight {
        address: String,
        height: Option<u64>,
    },
}

#[cw_serde]
pub struct VotingPowerAtHeightResponse {
    pub power: Uint128,
    pub height: u64,
}

#[cw_serde]
pub struct DaoDao(pub Addr);

impl DaoDao {
    pub fn get_voting_power(
        &self,
        querier: &QuerierWrapper,
        address: String,
    ) -> StdResult<VotingPowerAtHeightResponse> {
        let pair_info: VotingPowerAtHeightResponse = querier.query_wasm_smart(
            self.0.to_string(),
            &QueryMsg::VotingPowerAtHeight {
                address,
                height: None,
            },
        )?;
        Ok(pair_info)
    }
}
