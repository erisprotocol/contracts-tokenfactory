use std::vec;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, Decimal, StdResult, WasmMsg};
use kujira::{fin::ExecuteMsg, msg::KujiraMsg};

#[cw_serde]
pub struct Fin(pub Addr);

impl Fin {
    pub fn swap_msg(
        &self,
        offer_asset: &Coin,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> StdResult<CosmosMsg<KujiraMsg>> {
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.0.to_string(),
            funds: vec![offer_asset.clone()],
            msg: to_binary(&ExecuteMsg::Swap {
                offer_asset: Some(offer_asset.clone()),
                belief_price: belief_price.map(|a| a.into()),
                max_spread: max_spread.map(|a| a.into()),
                to: None,
            })?,
        }))
    }
}

#[test]
pub fn test_swap_msg() {
    use cosmwasm_std::Uint128;
    use std::str::FromStr;

    let coin = Coin {
        amount: Uint128::new(123),
        denom: "denom".to_string(),
    };

    assert_eq!(
        Fin(Addr::unchecked("fin"))
            .swap_msg(&coin, Some(Decimal::from_str("2.3").unwrap()), Some(Decimal::percent(10)))
            .unwrap(),
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "fin".to_string(),
            funds: vec![coin.clone()],
            msg: to_binary(&ExecuteMsg::Swap {
                offer_asset: Some(coin.clone()),
                belief_price: Some(Decimal::from_str("2.3").unwrap().into()),
                max_spread: Some(Decimal::percent(10).into()),
                to: None,
            })
            .unwrap(),
        })
    );
}
