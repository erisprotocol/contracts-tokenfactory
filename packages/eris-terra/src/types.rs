use crate::custom_execute_msg::CustomExecuteMsg;
use astroport::asset::{Asset, AssetInfo};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Coin, StdResult, Uint128};
use eris_chain_shared::{alliance_query::AllianceQueryWrapper, chain_trait::Validateable};

pub use astroport::asset::AssetInfoExt;

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
    Eris {
        addr: Addr,
    },
    Dex {
        addr: Addr,
    },
    Manta {
        addr: Addr,
        msg: MantaMsg,
    },
}

#[cw_serde]
pub enum MultiSwapRouterType {
    Manta {
        addr: Addr,
        msg: MantaMsg,
    },
}

#[cw_serde]
pub struct MantaMsg {
    pub swap: MantaSwap,
}

#[cw_serde]
pub struct MantaSwap {
    pub stages: Vec<Vec<(String, String)>>,
    pub min_return: Vec<Coin>,
}

impl StageType {
    pub fn dex(addr: &str) -> Self {
        Self::Dex {
            addr: Addr::unchecked(addr),
        }
    }
}

pub type DenomType = AssetInfo;
pub type CoinType = Asset;
pub type CustomMsgType = CustomExecuteMsg;
pub type CustomQueryType = AllianceQueryWrapper;

pub fn get_asset(info: DenomType, amount: Uint128) -> CoinType {
    Asset {
        info,
        amount,
    }
}

#[cw_serde]
pub struct HubChainConfigInput {}

impl Validateable<HubChainConfig> for HubChainConfigInput {
    fn validate(&self, _api: &dyn Api) -> StdResult<HubChainConfig> {
        Ok(HubChainConfig {})
    }
}

#[cw_serde]
pub struct HubChainConfig {}
