use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, CosmosMsg, Empty, StdError, StdResult, Uint128};
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

pub trait AssetInfoExt {
    /// simplifies converting an AssetInfo to an Asset with balance
    fn with_balance(&self, balance: Uint128) -> Asset;
}

impl AssetInfoExt for AssetInfo {
    fn with_balance(&self, amount: Uint128) -> Asset {
        match self {
            cw_asset::AssetInfoBase::Native(denom) => Asset::native(denom, amount),
            cw_asset::AssetInfoBase::Cw20(contract_addr) => {
                Asset::cw20(contract_addr.clone(), amount)
            },
            _ => todo!(),
        }
    }
}

pub trait AssetExt {
    /// simplifies converting an AssetInfo to an Asset with balance
    fn into_msg(self, receiver: &Addr) -> StdResult<CosmosMsg>;
}

impl AssetExt for Asset {
    fn into_msg(self, receiver: &Addr) -> StdResult<CosmosMsg> {
        self.transfer_msg(receiver).map_err(|e| StdError::generic_err(format!("asset: {0}", e)))
    }
}
