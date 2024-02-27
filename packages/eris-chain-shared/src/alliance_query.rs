use cosmwasm_std::CustomQuery;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type AllianceQueryWrapper = AllianceQuery;

// implement custom query
impl CustomQuery for AllianceQuery {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AllianceQuery {
    Delegation {
        denom: String,
        delegator: String,
        validator: String,
    },
}
