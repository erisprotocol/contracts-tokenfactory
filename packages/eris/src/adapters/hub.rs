use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coin, to_json_binary, Addr, CosmosMsg, Decimal, StdResult, VoteOption, WasmMsg,
};
use eris_chain_adapter::types::CustomMsgType;

use crate::hub::ExecuteMsg;

#[cw_serde]
pub struct Hub(pub Addr);

impl Hub {
    pub fn bond_msg(
        &self,
        denom: impl Into<String>,
        amount: u128,
        receiver: Option<String>,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_json_binary(&ExecuteMsg::Bond {
                receiver,
            })?,
            funds: vec![coin(amount, denom)],
        }))
    }

    pub fn vote_msg(
        &self,
        proposal_id: u64,
        vote: VoteOption,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_json_binary(&ExecuteMsg::Vote {
                proposal_id,
                vote,
            })?,
            funds: vec![],
        }))
    }

    pub fn vote_weighted_msg(
        &self,
        proposal_id: u64,
        votes: Vec<(Decimal, VoteOption)>,
    ) -> StdResult<CosmosMsg<CustomMsgType>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_json_binary(&&ExecuteMsg::VoteWeighted {
                proposal_id,
                votes,
            })?,
            funds: vec![],
        }))
    }
}
