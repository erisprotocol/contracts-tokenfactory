use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty};

#[cw_serde]
pub enum WithdrawType {
    Dex {
        addr: Addr,
    },
}

#[cw_serde]
pub enum StageType {
    Dex {
        addr: Addr,
    },
}

pub type DenomType = String;

pub type CustomMsgType = Empty;
