use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, CosmosMsg, Decimal, QuerierWrapper, StdResult, WasmMsg};
use cw_asset::AssetInfo;
use std::collections::{HashMap, HashSet};

#[cw_serde]
pub enum QueryMsg {
    WhitelistedAssets {},
}

pub type WhitelistedAssetsResponse = HashMap<String, Vec<AssetInfo>>;

#[cw_serde]
pub enum ExecuteMsg {
    SetAssetRewardDistribution(Vec<AssetDistribution>),
}

#[cw_serde]
pub struct AssetDistribution {
    pub asset: AssetInfo,
    pub distribution: Decimal,
}

#[cw_serde]
pub struct RestakeHub(pub Addr);

impl RestakeHub {
    pub fn vote_msg(&self, assets: Vec<AssetDistribution>) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            msg: to_json_binary(&ExecuteMsg::SetAssetRewardDistribution(assets))?,
            funds: vec![],
        }))
    }

    pub fn get_whitelisted_assets(&self, querier: &QuerierWrapper) -> StdResult<HashSet<String>> {
        let response: WhitelistedAssetsResponse =
            querier.query_wasm_smart(self.0.to_string(), &QueryMsg::WhitelistedAssets {})?;

        let mut result = HashSet::new();

        for (_, entry) in response {
            for element in entry {
                result.insert(element.to_string());
            }
        }

        Ok(result)
    }
}
