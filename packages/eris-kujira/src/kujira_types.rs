use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult};
use eris_chain_shared::chain_trait::Validateable;
use kujira::{denom::Denom, msg::KujiraMsg};

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

#[cw_serde]
pub struct HubChainConfigInput {}

impl Validateable<HubChainConfig> for HubChainConfigInput {
    fn validate(&self, _api: &dyn Api) -> StdResult<HubChainConfig> {
        Ok(HubChainConfig {})
    }
}

#[cw_serde]
pub struct HubChainConfig {}
