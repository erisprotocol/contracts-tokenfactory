use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, StakingMsg};
use eris_chain_adapter::types::CustomMsgType;

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Delegation {
    pub validator: String,
    pub amount: u128,
    pub denom: String,
}

impl Delegation {
    pub fn new(validator: &str, amount: u128, denom: impl Into<String>) -> Self {
        Self {
            validator: validator.to_string(),
            amount,
            denom: denom.into(),
        }
    }

    pub fn to_cosmos_msg(&self) -> CosmosMsg<CustomMsgType> {
        CosmosMsg::Staking(StakingMsg::Delegate {
            validator: self.validator.clone(),
            amount: Coin::new(self.amount, self.denom.clone()),
        })
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct SendFee {
    pub to_address: String,
    pub amount: u128,
    pub denom: String,
}

impl SendFee {
    pub fn new(to_address: Addr, amount: u128, denom: impl Into<String>) -> Self {
        Self {
            to_address: to_address.to_string(),
            amount,
            denom: denom.into(),
        }
    }

    pub fn to_cosmos_msg(&self) -> CosmosMsg<CustomMsgType> {
        CosmosMsg::Bank(BankMsg::Send {
            to_address: self.to_address.clone(),
            amount: vec![Coin::new(self.amount, self.denom.clone())],
        })
    }
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Undelegation {
    pub validator: String,
    pub amount: u128,
    pub denom: String,
}

impl Undelegation {
    pub fn new(validator: &str, amount: u128, denom: impl Into<String>) -> Self {
        Self {
            validator: validator.to_string(),
            amount,
            denom: denom.into(),
        }
    }

    pub fn to_cosmos_msg(&self) -> CosmosMsg<CustomMsgType> {
        CosmosMsg::Staking(StakingMsg::Undelegate {
            validator: self.validator.clone(),
            amount: Coin::new(self.amount, self.denom.clone()),
        })
    }
}

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Redelegation {
    pub src: String,
    pub dst: String,
    pub amount: u128,
    pub denom: String,
}

impl Redelegation {
    pub fn new(src: &str, dst: &str, amount: u128, denom: impl Into<String>) -> Self {
        Self {
            src: src.to_string(),
            dst: dst.to_string(),
            amount,
            denom: denom.into(),
        }
    }

    pub fn to_cosmos_msg(&self) -> CosmosMsg<CustomMsgType> {
        CosmosMsg::Staking(StakingMsg::Redelegate {
            src_validator: self.src.clone(),
            dst_validator: self.dst.clone(),
            amount: Coin::new(self.amount, self.denom.clone()),
        })
    }
}
