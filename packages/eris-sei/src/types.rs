use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Api, Coin, StdResult};
use eris_chain_shared::chain_trait::Validateable;
use sei_cosmwasm::{SeiMsg, SeiQueryWrapper};

#[cw_serde]
pub enum WithdrawType {}

#[cw_serde]
pub enum StageType {}

pub type DenomType = String;
pub type CustomMsgType = SeiMsg;
pub type CustomQueryType = SeiQueryWrapper;
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
