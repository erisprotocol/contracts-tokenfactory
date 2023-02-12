use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use kujira::{
    asset::Asset,
    fin::{ConfigResponse, QueryMsg, SimulationResponse},
};

#[cw_serde]
pub struct Fin(pub Addr);

impl Fin {
    pub fn query_config(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let res: ConfigResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.0.to_string(),
            msg: to_binary(&QueryMsg::Config {})?,
        }))?;

        Ok(res)
    }

    pub fn simulate(
        &self,
        querier: &QuerierWrapper,
        offer_asset: &Asset,
    ) -> StdResult<SimulationResponse> {
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.0.to_string(),
            msg: to_binary(&QueryMsg::Simulation {
                offer_asset: offer_asset.clone(),
            })?,
        }))
    }
}
