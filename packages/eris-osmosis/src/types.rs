use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Api, Coin, Empty, StdResult};
use eris_chain_shared::chain_trait::Validateable;

#[cw_serde]
pub enum WithdrawType {
    Withdraw {
        pool_id: u64,
        token_out_mins: Vec<Coin>,
    },
}

#[cw_serde]
pub enum StageType {
    Osmo {
        pool_id: u64,
        token_out_denom: String,
    },
}

pub type DenomType = String;
pub type CustomMsgType = Empty;
pub type CoinType = Coin;

#[cw_serde]
pub struct HubChainConfigInput {}

impl Validateable<HubChainConfig> for HubChainConfigInput {
    fn validate(&self, _api: &dyn Api) -> StdResult<HubChainConfig> {
        Ok(HubChainConfig {})
    }
}

#[cw_serde]
pub struct HubChainConfig {}
