use astroport::querier::{query_supply, query_token_balance};
use cosmwasm_std::{to_binary, Addr, CosmosMsg, QuerierWrapper, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

pub struct Token(pub Addr);

impl Token {
    pub fn mint(&self, amount: Uint128, receiver: Addr) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: receiver.to_string(),
                amount,
            })?,
            funds: vec![],
        }))
    }

    pub fn burn(&self, amount: Uint128) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount,
            })?,
            funds: vec![],
        }))
    }

    pub fn query_amount(&self, querier: &QuerierWrapper, account: Addr) -> StdResult<Uint128> {
        query_token_balance(querier, self.0.to_string(), account)
    }

    pub fn query_supply(&self, querier: &QuerierWrapper) -> StdResult<Uint128> {
        query_supply(querier, self.0.to_string())
    }
}
