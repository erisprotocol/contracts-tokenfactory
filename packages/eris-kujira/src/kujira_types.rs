use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, StdResult};
use eris_chain_shared::chain_trait::Validateable;
use kujira::{denom::Denom, msg::KujiraMsg};

use crate::adapters::fin_multi::FinMulti;

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
pub struct HubChainConfigInput {
    pub fin_multi: String,
}

impl Validateable<HubChainConfig> for HubChainConfigInput {
    fn validate(&self, api: &dyn Api) -> StdResult<HubChainConfig> {
        Ok(HubChainConfig {
            fin_multi: FinMulti(api.addr_validate(&self.fin_multi)?),
        })
    }
}

#[cw_serde]
pub struct HubChainConfig {
    pub fin_multi: FinMulti,
}
