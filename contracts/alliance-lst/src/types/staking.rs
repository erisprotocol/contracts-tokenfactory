use cosmwasm_std::{Addr, BankMsg, Binary, CosmosMsg};
use eris_chain_adapter::types::CustomMsgType;

use terra_proto_rs::{
    alliance::alliance::{MsgClaimDelegationRewards, MsgDelegate, MsgRedelegate, MsgUndelegate},
    cosmos::base::v1beta1::Coin,
    prost::Message,
};

pub fn withdraw_delegator_reward_msg(
    delegator_address: String,
    validator_address: String,
    denom: String,
) -> CosmosMsg<CustomMsgType> {
    let msg = MsgClaimDelegationRewards {
        delegator_address,
        validator_address,
        denom,
    };
    CosmosMsg::Stargate {
        type_url: "/alliance.alliance.MsgClaimDelegationRewards".to_string(),
        value: Binary::from(msg.encode_to_vec()),
    }
}

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

    pub fn to_cosmos_msg(&self, delegator: String) -> CosmosMsg<CustomMsgType> {
        let delegate_msg = MsgDelegate {
            amount: Some(Coin {
                denom: self.denom.clone(),
                amount: self.amount.to_string(),
            }),
            delegator_address: delegator,
            validator_address: self.validator.clone(),
        };
        CosmosMsg::Stargate {
            type_url: "/alliance.alliance.MsgDelegate".to_string(),
            value: Binary::from(delegate_msg.encode_to_vec()),
        }
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
            amount: vec![cosmwasm_std::Coin::new(self.amount, self.denom.clone())],
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

    pub fn to_cosmos_msg(&self, delegator: String) -> CosmosMsg<CustomMsgType> {
        let undelegate_msg = MsgUndelegate {
            amount: Some(Coin {
                denom: self.denom.clone(),
                amount: self.amount.to_string(),
            }),
            delegator_address: delegator,
            validator_address: self.validator.to_string(),
        };
        CosmosMsg::Stargate {
            type_url: "/alliance.alliance.MsgUndelegate".to_string(),
            value: Binary::from(undelegate_msg.encode_to_vec()),
        }
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

    pub fn to_cosmos_msg(&self, delegator: String) -> CosmosMsg<CustomMsgType> {
        let redelegate_msg = MsgRedelegate {
            amount: Some(Coin {
                denom: self.denom.clone(),
                amount: self.amount.to_string(),
            }),
            delegator_address: delegator,
            validator_src_address: self.src.to_string(),
            validator_dst_address: self.dst.to_string(),
        };
        CosmosMsg::Stargate {
            type_url: "/alliance.alliance.MsgRedelegate".to_string(),
            value: Binary::from(redelegate_msg.encode_to_vec()),
        }
    }
}
