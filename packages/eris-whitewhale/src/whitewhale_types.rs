use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Empty, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};
use eris_chain_shared::chain_trait::Validateable;

#[cw_serde]
pub enum WithdrawType {
    Dex {
        addr: Addr,
    },
}

impl WithdrawType {
    pub fn dex(addr: &str) -> Self {
        Self::Dex {
            addr: Addr::unchecked(addr),
        }
    }
}

#[cw_serde]
pub enum StageType {
    Dex {
        addr: Addr,
    },
}

impl StageType {
    pub fn dex(addr: &str) -> Self {
        Self::Dex {
            addr: Addr::unchecked(addr),
        }
    }
}

pub type DenomType = AssetInfo;
pub type CustomMsgType = Empty;
pub type CoinType = Asset;
pub type CustomQueryType = Empty;

#[cw_serde]
pub struct HubChainConfigInput {}

impl Validateable<HubChainConfig> for HubChainConfigInput {
    fn validate(&self, _api: &dyn Api) -> StdResult<HubChainConfig> {
        Ok(HubChainConfig {})
    }
}
#[cw_serde]
pub struct HubChainConfig {}

pub fn get_asset(info: AssetInfo, amount: Uint128) -> Asset {
    Asset {
        info,
        amount,
    }
}
