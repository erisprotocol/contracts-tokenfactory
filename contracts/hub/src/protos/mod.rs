use cosmwasm_std::{Binary, CosmosMsg};
use eris_chain_adapter::types::CustomMsgType;
use protobuf::Message;

use self::proto::MsgVoteWeighted;

pub mod proto;

impl MsgVoteWeighted {
    pub fn to_cosmos_msg(&self) -> CosmosMsg<CustomMsgType> {
        let exec_bytes: Vec<u8> = self.write_to_bytes().unwrap();

        CosmosMsg::Stargate {
            type_url: "/cosmos.gov.v1beta1.MsgVoteWeighted".to_string(),
            value: Binary::from(exec_bytes),
        }
    }
}
