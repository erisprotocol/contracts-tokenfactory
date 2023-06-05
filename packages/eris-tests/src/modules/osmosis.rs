use anyhow::Result as AnyResult;
use cosmwasm_schema::{schemars, serde};
use cosmwasm_std::{attr, Addr, BankMsg, CosmosMsg, Decimal, Empty, Event, Uint128};

use cw_multi_test::{AppResponse, BankSudo, CosmosRouter, Module, SudoMsg};
use eris_chain_adapter::types::CustomMsgType;

pub struct OsmosisModule {}

impl OsmosisModule {}

impl Module for OsmosisModule {
    type ExecT = CustomMsgType;

    type QueryT = Empty;

    type SudoT = Empty;

    fn execute<ExecC, QueryC>(
        &self,
        api: &dyn cosmwasm_std::Api,
        storage: &mut dyn cosmwasm_std::Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &cosmwasm_std::BlockInfo,
        sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: std::fmt::Debug
            + Clone
            + PartialEq
            + schemars::JsonSchema
            + serde::de::DeserializeOwned
            + 'static,
        QueryC: cosmwasm_std::CustomQuery + serde::de::DeserializeOwned + 'static,
    {
        // match msg {
        //     CustomMsgType::Auth(_) => todo!(),
        //     CustomMsgType::Denom(d) => match d {
        //         DenomMsg::Create {
        //             subdenom,
        //         } => {
        //             storage.set(subdenom.as_bytes(), &Uint128::zero().to_be_bytes());

        //             Ok(AppResponse {
        //                 events: vec![],
        //                 data: None,
        //             })
        //         },
        //         DenomMsg::Mint {
        //             amount,
        //             denom,
        //             recipient,
        //         } => {
        //             let mut supply = storage
        //                 .get(denom.as_bytes())
        //                 .map(|bz| u128::from_be_bytes(bz.try_into().unwrap()))
        //                 .map(Uint128::from)
        //                 .unwrap_or_default();

        //             supply += amount;
        //             storage.set(denom.as_bytes(), &supply.to_be_bytes());
        //             router.sudo(
        //                 api,
        //                 storage,
        //                 block,
        //                 SudoMsg::Bank(BankSudo::Mint {
        //                     to_address: recipient.to_string(),
        //                     amount: denom.coins(&amount),
        //                 }),
        //             )?;
        //             Ok(AppResponse {
        //                 events: vec![Event::new("mint").add_attributes(vec![
        //                     attr("amount", amount),
        //                     attr("denom", denom.to_string()),
        //                     attr("recipient", recipient),
        //                 ])],
        //                 data: None,
        //             })
        //         },
        //         DenomMsg::Burn {
        //             denom,
        //             amount,
        //         } => {
        //             let mut supply = storage
        //                 .get(denom.as_bytes())
        //                 .map(|bz| u128::from_be_bytes(bz.try_into().unwrap()))
        //                 .map(Uint128::from)
        //                 .unwrap_or_default();
        //             println!("Burning {} of {}", amount, denom);
        //             supply -= amount;
        //             storage.set(denom.as_bytes(), &supply.to_be_bytes());

        //             router.execute(
        //                 api,
        //                 storage,
        //                 block,
        //                 sender,
        //                 CosmosMsg::Bank(BankMsg::Burn {
        //                     amount: denom.coins(&amount),
        //                 }),
        //             )?;

        //             Ok(AppResponse {
        //                 events: vec![Event::new("burn").add_attributes(vec![
        //                     attr("amount", amount),
        //                     attr("denom", denom.to_string()),
        //                 ])],
        //                 data: None,
        //             })
        //         },
        //         _ => todo!(),
        //     },
        // }
        todo!()
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn cosmwasm_std::Api,
        _storage: &mut dyn cosmwasm_std::Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &cosmwasm_std::BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: std::fmt::Debug
            + Clone
            + PartialEq
            + schemars::JsonSchema
            + serde::de::DeserializeOwned
            + 'static,
        QueryC: cosmwasm_std::CustomQuery + serde::de::DeserializeOwned + 'static,
    {
        todo!()
    }

    fn query(
        &self,
        _api: &dyn cosmwasm_std::Api,
        _storage: &dyn cosmwasm_std::Storage,
        _querier: &dyn cosmwasm_std::Querier,
        _block: &cosmwasm_std::BlockInfo,
        _request: Self::QueryT,
    ) -> AnyResult<cosmwasm_std::Binary> {
        todo!()
        // match request {
        //     KujiraQuery::Bank(b) => match b {
        //         BankQuery::Supply {
        //             denom,
        //         } => {
        //             let supply = storage
        //                 .get(denom.as_bytes())
        //                 .map(|bz| u128::from_be_bytes(bz.try_into().unwrap()))
        //                 .unwrap_or_default();

        //             Ok(to_binary(&SupplyResponse {
        //                 amount: denom.coin(&Uint128::from(supply)),
        //             })?)
        //         },
        //     },
        //     KujiraQuery::Oracle(o) => match o {
        //         OracleQuery::ExchangeRate {
        //             ..
        //         } => Ok(to_binary(&ExchangeRateResponse {
        //             rate: self.oracle_price,
        //         })?),
        //     },
        // }
    }
}
