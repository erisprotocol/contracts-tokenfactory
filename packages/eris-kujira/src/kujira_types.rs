use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, BankMsg, Coin, CosmosMsg, Empty, StdResult, Uint128};
use eris_chain_shared::chain_trait::Validateable;
use kujira::{
    asset::{Asset, AssetInfo},
    denom::Denom,
    msg::KujiraMsg,
};

#[cw_serde]
pub enum WithdrawType {
    BlackWhale {
        addr: Addr,
    },
    Bow {
        addr: Addr,
    },
}

impl WithdrawType {
    pub fn bw(addr: &str) -> Self {
        Self::BlackWhale {
            addr: Addr::unchecked(addr),
        }
    }

    pub fn bow(addr: &str) -> Self {
        Self::Bow {
            addr: Addr::unchecked(addr),
        }
    }
}

#[cw_serde]
pub enum StageType {
    Fin {
        addr: Addr,
    },
}

impl StageType {
    pub fn fin(addr: &str) -> Self {
        Self::Fin {
            addr: Addr::unchecked(addr),
        }
    }
}

pub type DenomType = Denom;
pub type CustomMsgType = KujiraMsg;
pub type CoinType = Coin;
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

pub trait AssetInfoExt {
    /// simplifies converting an AssetInfo to an Asset with balance
    fn with_balance(&self, balance: Uint128) -> Asset;
}

impl AssetInfoExt for AssetInfo {
    fn with_balance(&self, amount: Uint128) -> Asset {
        match self {
            AssetInfo::NativeToken {
                denom,
            } => Asset {
                info: AssetInfo::NativeToken {
                    denom: denom.clone(),
                },
                amount,
            },
        }
    }
}

pub trait AssetExt {
    /// simplifies converting an AssetInfo to an Asset with balance
    fn into_msg(self, receiver: &Addr) -> StdResult<CosmosMsg>;
}

impl AssetExt for Asset {
    fn into_msg(self, receiver: &Addr) -> StdResult<CosmosMsg> {
        match self.info {
            AssetInfo::NativeToken {
                denom,
            } => Ok(CosmosMsg::Bank(BankMsg::Send {
                to_address: receiver.into(),
                amount: vec![Coin {
                    denom: denom.to_string(),
                    amount: self.amount,
                }],
            })),
        }
    }
}
