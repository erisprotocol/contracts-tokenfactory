use astroport::asset::{token_asset_info, AssetInfo, PairInfo};
use astroport::factory::FeeInfoResponse;
use astroport::pair::QueryMsg::{Pair, Simulation};
use astroport::pair::SimulationResponse;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: Default::default(),
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            },
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg,
            }) => {
                if contract_addr == "factory" {
                    match from_json(msg).unwrap() {
                        astroport::factory::QueryMsg::FeeInfo {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&FeeInfoResponse {
                                fee_address: Some(Addr::unchecked("fee_address")),
                                total_fee_bps: 30,
                                maker_fee_bps: 1660,
                            })
                            .into(),
                        ),
                        astroport::factory::QueryMsg::Pair {
                            asset_infos,
                        } => {
                            if asset_infos.contains(&token_asset_info(Addr::unchecked("unknown"))) {
                                SystemResult::Ok(ContractResult::Err("unknown pair".to_string()))
                            } else {
                                SystemResult::Ok(
                                    to_json_binary(&PairInfo {
                                        asset_infos,
                                        contract_addr: Addr::unchecked("pair-x"),
                                        liquidity_token: Addr::unchecked("lp-x"),
                                        pair_type: astroport::factory::PairType::Xyk {},
                                    })
                                    .into(),
                                )
                            }
                        },
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == "router" {
                    match from_json(msg).unwrap() {
                        astroport::router::QueryMsg::SimulateSwapOperations {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&astroport::router::SimulateSwapOperationsResponse {
                                amount: Uint128::from(1000000u128),
                            })
                            .into(),
                        ),

                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == "pair_contract" {
                    match from_json(msg).unwrap() {
                        Pair {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&PairInfo {
                                asset_infos: vec![
                                    {
                                        AssetInfo::Token {
                                            contract_addr: Addr::unchecked("token"),
                                        }
                                    },
                                    {
                                        AssetInfo::NativeToken {
                                            denom: "uluna".to_string(),
                                        }
                                    },
                                ],
                                contract_addr: Addr::unchecked("pair_contract"),
                                liquidity_token: Addr::unchecked("liquidity_token"),
                                pair_type: astroport::factory::PairType::Xyk {},
                            })
                            .into(),
                        ),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == "pair0001" {
                    match from_json(msg).unwrap() {
                        Pair {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&PairInfo {
                                asset_infos: vec![
                                    {
                                        AssetInfo::Token {
                                            contract_addr: Addr::unchecked("any"),
                                        }
                                    },
                                    {
                                        AssetInfo::NativeToken {
                                            denom: "uluna".to_string(),
                                        }
                                    },
                                ],
                                contract_addr: Addr::unchecked("pair0001"),
                                liquidity_token: Addr::unchecked("liquidity_token_1"),
                                pair_type: astroport::factory::PairType::Xyk {},
                            })
                            .into(),
                        ),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == "pair0002" {
                    match from_json(msg).unwrap() {
                        Pair {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&PairInfo {
                                asset_infos: vec![
                                    {
                                        AssetInfo::NativeToken {
                                            denom: "uluna".to_string(),
                                        }
                                    },
                                    {
                                        AssetInfo::NativeToken {
                                            denom: "ibc/token".to_string(),
                                        }
                                    },
                                ],
                                contract_addr: Addr::unchecked("pair0002"),
                                liquidity_token: Addr::unchecked("liquidity_token_2"),
                                pair_type: astroport::factory::PairType::Xyk {},
                            })
                            .into(),
                        ),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == "pair_contract_2" {
                    match from_json(msg).unwrap() {
                        Pair {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&PairInfo {
                                asset_infos: vec![
                                    {
                                        AssetInfo::NativeToken {
                                            denom: "uluna".to_string(),
                                        }
                                    },
                                    {
                                        AssetInfo::NativeToken {
                                            denom: "ibc/token".to_string(),
                                        }
                                    },
                                ],
                                contract_addr: Addr::unchecked("pair_contract_2"),
                                liquidity_token: Addr::unchecked("liquidity_token_3"),
                                pair_type: astroport::factory::PairType::Xyk {},
                            })
                            .into(),
                        ),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == "pair_astro_token" {
                    match from_json(msg).unwrap() {
                        Pair {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&PairInfo {
                                asset_infos: vec![
                                    {
                                        AssetInfo::Token {
                                            contract_addr: Addr::unchecked("astro"),
                                        }
                                    },
                                    {
                                        AssetInfo::Token {
                                            contract_addr: Addr::unchecked("token"),
                                        }
                                    },
                                ],
                                contract_addr: Addr::unchecked("pair_astro_token"),
                                liquidity_token: Addr::unchecked("astro_token_lp"),
                                pair_type: astroport::factory::PairType::Xyk {},
                            })
                            .into(),
                        ),
                        Simulation {
                            ..
                        } => SystemResult::Ok(
                            to_json_binary(&SimulationResponse {
                                return_amount: Uint128::from(1000000u128),
                                commission_amount: Uint128::zero(),
                                spread_amount: Uint128::zero(),
                            })
                            .into(),
                        ),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    match from_json(msg).unwrap() {
                        Cw20QueryMsg::TokenInfo {} => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::Unknown {});
                                    },
                                };

                            let mut total_supply = Uint128::zero();

                            for balance in balances {
                                total_supply += *balance.1;
                            }

                            SystemResult::Ok(
                                to_json_binary(&TokenInfoResponse {
                                    name: "mAPPL".to_string(),
                                    symbol: "mAPPL".to_string(),
                                    decimals: 6,
                                    total_supply,
                                })
                                .into(),
                            )
                        },
                        Cw20QueryMsg::Balance {
                            address,
                        } => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::Unknown {});
                                    },
                                };

                            let balance = match balances.get(&address) {
                                Some(v) => v,
                                None => {
                                    return SystemResult::Err(SystemError::Unknown {});
                                },
                            };

                            SystemResult::Ok(
                                to_json_binary(&BalanceResponse {
                                    balance: *balance,
                                })
                                .into(),
                            )
                        },
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                }
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }

    /*// configure the token owner mock querier
    pub fn with_tax(&mut self, rate: Decimal, caps: &[(&String, &Uint128)]) {
        self.tax_querier = TaxQuerier::new(rate, caps);
    }*/

    pub fn with_balance(&mut self, balances: &[(&String, &[Coin])]) {
        for (addr, balance) in balances {
            self.base.update_balance(addr.to_string(), balance.to_vec());
        }
    }
}
