use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CustomMsg, Uint128};

// see https://github.com/terra-money/alliance-protocol/blob/main/packages/alliance-protocol/src/token_factory.rs
#[cw_serde]
pub enum CustomExecuteMsg {
    Token(TokenExecuteMsg),
}

impl CustomMsg for CustomExecuteMsg {}

#[cw_serde]
pub enum TokenExecuteMsg {
    CreateDenom {
        subdenom: String,
    },
    MintTokens {
        denom: String,
        amount: Uint128,
        mint_to_address: String,
    },
    BurnTokens {
        denom: String,
        amount: Uint128,
        burn_from_address: String,
    },
}
