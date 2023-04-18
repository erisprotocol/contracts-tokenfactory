use anyhow::Result;
use cosmwasm_std::{coin, coins, Addr, Delegation, FullDelegation, StdResult, VoteOption};
use cw_multi_test::{AppResponse, Executor};
use eris::{
    // arb_vault::{LsdConfig, LsdType},
    emp_gauges::AddEmpInfo,
    governance_helper::WEEK,
};

use crate::base::{BaseErisTestInitMessage, BaseErisTestPackage, CustomApp};

pub const MULTIPLIER: u64 = 1000000;

pub struct EscrowHelper {
    pub owner: Addr,
    pub base: BaseErisTestPackage,
}

impl EscrowHelper {
    pub fn init(router_ref: &mut CustomApp, use_default_hub: bool) -> Self {
        let owner = Addr::unchecked("owner");
        Self {
            owner: owner.clone(),
            base: BaseErisTestPackage::init_all(
                router_ref,
                BaseErisTestInitMessage {
                    owner,
                    use_uniform_hub: use_default_hub,
                },
            ),
        }
    }

    pub fn get_ustake_addr(&self) -> Addr {
        self.base.amp_token.get_address()
    }

    // pub fn get_stader_token_addr(&self) -> Addr {
    //     self.base.stader_token.get_address()
    // }

    // pub fn get_steak_token_addr(&self) -> Addr {
    //     self.base.steak_token.get_address()
    // }

    pub fn emp_tune(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        router_ref.execute_contract(
            self.owner.clone(),
            self.base.emp_gauges.get_address(),
            &eris::emp_gauges::ExecuteMsg::TuneEmps {},
            &[],
        )
    }

    pub fn emp_add_points(
        &self,
        router_ref: &mut CustomApp,
        emps: Vec<AddEmpInfo>,
    ) -> Result<AppResponse> {
        self.emp_execute(
            router_ref,
            eris::emp_gauges::ExecuteMsg::AddEmps {
                emps,
            },
        )
    }

    pub fn emp_execute(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::emp_gauges::ExecuteMsg,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            self.owner.clone(),
            self.base.emp_gauges.get_address(),
            &execute,
            &[],
        )
    }

    pub fn emp_execute_sender(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::emp_gauges::ExecuteMsg,
        sender: impl Into<String>,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.emp_gauges.get_address(),
            &execute,
            &[],
        )
    }

    pub fn emp_query_config(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::emp_gauges::ConfigResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.emp_gauges.get_address_string(),
            &eris::emp_gauges::QueryMsg::Config {},
        )
    }

    pub fn emp_query_validator_history(
        &self,
        router_ref: &mut CustomApp,
        validator_addr: impl Into<String>,
        period: u64,
    ) -> StdResult<eris::emp_gauges::VotedValidatorInfoResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.emp_gauges.get_address_string(),
            &eris::emp_gauges::QueryMsg::ValidatorInfoAtPeriod {
                validator_addr: validator_addr.into(),
                period,
            },
        )
    }

    pub fn emp_query_tune_info(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::emp_gauges::GaugeInfoResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.emp_gauges.get_address_string(),
            &eris::emp_gauges::QueryMsg::TuneInfo {},
        )
    }

    pub fn amp_tune(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        router_ref.execute_contract(
            self.owner.clone(),
            self.base.amp_gauges.get_address(),
            &eris::amp_gauges::ExecuteMsg::TuneVamp {},
            &[],
        )
    }

    pub fn amp_vote(
        &self,
        router_ref: &mut CustomApp,
        user: impl Into<String>,
        votes: Vec<(String, u16)>,
    ) -> Result<AppResponse> {
        self.amp_execute_sender(
            router_ref,
            eris::amp_gauges::ExecuteMsg::Vote {
                votes,
            },
            user,
        )
    }

    pub fn amp_execute(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::amp_gauges::ExecuteMsg,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            self.owner.clone(),
            self.base.amp_gauges.get_address(),
            &execute,
            &[],
        )
    }

    pub fn amp_execute_sender(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::amp_gauges::ExecuteMsg,
        sender: impl Into<String>,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.amp_gauges.get_address(),
            &execute,
            &[],
        )
    }

    pub fn amp_query_config(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::amp_gauges::ConfigResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.amp_gauges.get_address_string(),
            &eris::amp_gauges::QueryMsg::Config {},
        )
    }

    pub fn amp_query_validator_history(
        &self,
        router_ref: &mut CustomApp,
        validator_addr: impl Into<String>,
        period: u64,
    ) -> StdResult<eris::amp_gauges::VotedValidatorInfoResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.amp_gauges.get_address_string(),
            &eris::amp_gauges::QueryMsg::ValidatorInfoAtPeriod {
                validator_addr: validator_addr.into(),
                period,
            },
        )
    }

    pub fn amp_query_tune_info(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::amp_gauges::GaugeInfoResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.amp_gauges.get_address_string(),
            &eris::amp_gauges::QueryMsg::TuneInfo {},
        )
    }

    pub fn mint_amp_token(&self, router_ref: &mut CustomApp, to: String, amount: u128) {
        router_ref
            .sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
                to_address: "bank".to_string(),
                amount: vec![coin(amount, self.base.amp_token.get_address_string())],
            }))
            .unwrap();

        router_ref
            .send_tokens(
                Addr::unchecked("bank"),
                Addr::unchecked(to),
                &coins(amount, self.base.amp_token.get_address_string()),
            )
            .unwrap();

        // let msg = cw20::Cw20ExecuteMsg::Mint {
        //     recipient: to.clone(),
        //     amount: Uint128::from(amount),
        // };
        // let res = router_ref
        //     .execute_contract(self.owner.clone(), self.base.amp_lp.get_address(), &msg, &[])
        //     .unwrap();
        // assert_eq!(res.events[1].attributes[1], attr("action", "mint"));
        // assert_eq!(res.events[1].attributes[2], attr("to", to));
        // assert_eq!(res.events[1].attributes[3], attr("amount", Uint128::from(amount)));
    }

    pub fn ve_lock_lp(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
        amount: u128,
        lock_time: u64,
    ) -> Result<AppResponse> {
        let sender: String = sender.into();
        self.mint_amp_token(router_ref, sender.clone(), amount);

        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.voting_escrow.get_address(),
            &eris::voting_escrow::ExecuteMsg::CreateLock {
                time: lock_time,
            },
            &[coin(amount, self.base.amp_token.get_address_string())],
        )
    }

    pub fn ve_add_funds_lock(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
        amount: u128,
        extend_to_min_periods: Option<bool>,
    ) -> Result<AppResponse> {
        let sender: String = sender.into();
        self.mint_amp_token(router_ref, sender.clone(), amount);

        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.voting_escrow.get_address(),
            &eris::voting_escrow::ExecuteMsg::ExtendLockAmount {
                extend_to_min_periods,
            },
            &[coin(amount, self.base.amp_token.get_address_string())],
        )
    }

    pub fn ve_add_funds_lock_for_user(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
        user: impl Into<String>,
        amount: u128,
    ) -> Result<AppResponse> {
        let sender: String = sender.into();
        self.mint_amp_token(router_ref, sender.clone(), amount);

        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.voting_escrow.get_address(),
            &eris::voting_escrow::ExecuteMsg::DepositFor {
                user: user.into(),
            },
            &[coin(amount, self.base.amp_token.get_address_string())],
        )
    }

    pub fn ve_extend_lock_time(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
        periods: u64,
    ) -> Result<AppResponse> {
        self.ve_execute_sender(
            router_ref,
            eris::voting_escrow::ExecuteMsg::ExtendLockTime {
                time: periods * WEEK,
            },
            Addr::unchecked(sender),
        )
    }

    pub fn ve_withdraw(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
    ) -> Result<AppResponse> {
        self.ve_execute_sender(
            router_ref,
            eris::voting_escrow::ExecuteMsg::Withdraw {},
            Addr::unchecked(sender),
        )
    }

    pub fn ve_execute(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::voting_escrow::ExecuteMsg,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            self.owner.clone(),
            self.base.voting_escrow.get_address(),
            &execute,
            &[],
        )
    }

    pub fn ve_execute_sender(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::voting_escrow::ExecuteMsg,
        sender: Addr,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(sender, self.base.voting_escrow.get_address(), &execute, &[])
    }

    pub fn hub_execute(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::hub::ExecuteMsg,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(self.owner.clone(), self.base.hub.get_address(), &execute, &[])
    }

    pub fn hub_submit_batch(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        self.hub_execute(router_ref, eris::hub::ExecuteMsg::SubmitBatch {})
    }

    pub fn hub_reconcile(&self, router_ref: &mut CustomApp, amount: u128) -> Result<AppResponse> {
        router_ref
            .sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
                to_address: self.base.hub.get_address_string(),
                amount: vec![coin(amount, "uluna")],
            }))
            .unwrap();

        self.hub_execute(router_ref, eris::hub::ExecuteMsg::Reconcile {})
    }

    pub fn hub_remove_validator(
        &self,
        router_ref: &mut CustomApp,
        validator_addr: impl Into<String>,
    ) -> Result<AppResponse> {
        self.hub_execute(
            router_ref,
            eris::hub::ExecuteMsg::RemoveValidator {
                validator: validator_addr.into(),
            },
        )
    }

    pub fn hub_harvest(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        self.hub_execute(
            router_ref,
            eris::hub::ExecuteMsg::Harvest {
                validators: None,
                stages: None,
                withdrawals: None,
            },
        )
    }

    pub fn hub_add_validator(
        &self,
        router_ref: &mut CustomApp,
        validator_addr: impl Into<String>,
    ) -> Result<AppResponse> {
        self.hub_execute(
            router_ref,
            eris::hub::ExecuteMsg::AddValidator {
                validator: validator_addr.into(),
            },
        )
    }

    pub fn hub_bond(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
        amount: u128,
        denom: impl Into<String>,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.hub.get_address(),
            &eris::hub::ExecuteMsg::Bond {
                receiver: None,
            },
            &[coin(amount, denom.into())],
        )
    }

    pub fn hub_allow_donate(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        self.hub_execute(
            router_ref,
            eris::hub::ExecuteMsg::UpdateConfig {
                protocol_fee_contract: None,
                protocol_reward_fee: None,
                allow_donations: Some(true),
                delegation_strategy: None,
                vote_operator: None,
                chain_config: None,
                default_max_spread: None,
                operator: None,
                stages_preset: None,
                withdrawals_preset: None,
            },
        )
    }

    pub fn hub_donate(
        &self,
        router_ref: &mut CustomApp,
        sender: impl Into<String>,
        amount: u128,
        denom: impl Into<String>,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.hub.get_address(),
            &eris::hub::ExecuteMsg::Donate {},
            &[coin(amount, denom.into())],
        )
    }

    pub fn hub_execute_sender(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::hub::ExecuteMsg,
        sender: Addr,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(sender, self.base.hub.get_address(), &execute, &[])
    }

    pub fn hub_rebalance(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        self.hub_execute(
            router_ref,
            eris::hub::ExecuteMsg::Rebalance {
                min_redelegation: None,
            },
        )
    }

    pub fn hub_tune(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
        self.hub_execute(router_ref, eris::hub::ExecuteMsg::TuneDelegations {})
    }

    pub fn hub_query_config(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::hub::ConfigResponse> {
        router_ref
            .wrap()
            .query_wasm_smart(self.base.hub.get_address_string(), &eris::hub::QueryMsg::Config {})
    }

    pub fn hub_query_wanted_delegations(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::hub::WantedDelegationsResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.hub.get_address_string(),
            &eris::hub::QueryMsg::WantedDelegations {},
        )
    }

    pub fn hub_query_state(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::hub::StateResponse> {
        router_ref
            .wrap()
            .query_wasm_smart(self.base.hub.get_address_string(), &eris::hub::QueryMsg::State {})
    }

    pub fn hub_query_all_delegations(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<Vec<Delegation>> {
        router_ref.wrap().query_all_delegations(self.base.hub.get_address_string())
    }

    pub fn hub_query_delegation(
        &self,
        router_ref: &mut CustomApp,
        validator: impl Into<String>,
    ) -> StdResult<Option<FullDelegation>> {
        router_ref.wrap().query_delegation(self.base.hub.get_address_string(), validator)
    }

    pub fn prop_vote(
        &self,
        router_ref: &mut CustomApp,
        user: impl Into<String>,
        proposal_id: u64,
        vote: VoteOption,
    ) -> Result<AppResponse> {
        self.prop_execute_sender(
            router_ref,
            eris::prop_gauges::ExecuteMsg::Vote {
                proposal_id,
                vote,
            },
            user,
        )
    }

    pub fn prop_init(
        &self,
        router_ref: &mut CustomApp,
        user: impl Into<String>,
        proposal_id: u64,
        end_time_s: u64,
    ) -> Result<AppResponse> {
        self.prop_execute_sender(
            router_ref,
            eris::prop_gauges::ExecuteMsg::InitProp {
                proposal_id,
                end_time_s,
            },
            user,
        )
    }

    pub fn prop_execute(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::prop_gauges::ExecuteMsg,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            self.owner.clone(),
            self.base.prop_gauges.get_address(),
            &execute,
            &[],
        )
    }

    pub fn prop_execute_sender(
        &self,
        router_ref: &mut CustomApp,
        execute: eris::prop_gauges::ExecuteMsg,
        sender: impl Into<String>,
    ) -> Result<AppResponse> {
        router_ref.execute_contract(
            Addr::unchecked(sender),
            self.base.prop_gauges.get_address(),
            &execute,
            &[],
        )
    }

    pub fn prop_query_config(
        &self,
        router_ref: &mut CustomApp,
    ) -> StdResult<eris::prop_gauges::ConfigResponse> {
        router_ref.wrap().query_wasm_smart(
            self.base.prop_gauges.get_address_string(),
            &eris::prop_gauges::QueryMsg::Config {},
        )
    }

    // pub fn arb_query_config(
    //     &self,
    //     router_ref: &mut CustomApp,
    // ) -> StdResult<eris::arb_vault::ConfigResponse> {
    //     router_ref.wrap().query_wasm_smart(
    //         self.base.arb_vault.get_address_string(),
    //         &eris::arb_vault::QueryMsg::Config {},
    //     )
    // }

    // pub fn arb_query_state(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     details: Option<bool>,
    // ) -> StdResult<eris::arb_vault::StateResponse> {
    //     router_ref.wrap().query_wasm_smart(
    //         self.base.arb_vault.get_address_string(),
    //         &eris::arb_vault::QueryMsg::State {
    //             details,
    //         },
    //     )
    // }

    // pub fn arb_query_user_info(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     address: impl Into<String>,
    // ) -> StdResult<eris::arb_vault::UserInfoResponse> {
    //     router_ref.wrap().query_wasm_smart(
    //         self.base.arb_vault.get_address_string(),
    //         &eris::arb_vault::QueryMsg::UserInfo {
    //             address: address.into(),
    //         },
    //     )
    // }

    // pub fn arb_execute(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     execute: eris::arb_vault::ExecuteMsg,
    // ) -> Result<AppResponse> {
    //     self.arb_execute_sender(router_ref, execute, self.owner.to_string())
    // }
    // pub fn arb_execute_whitelist(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     execute: eris::arb_vault::ExecuteMsg,
    // ) -> Result<AppResponse> {
    //     self.arb_execute_sender(router_ref, execute, "executor")
    // }

    // pub fn arb_execute_sender(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     execute: eris::arb_vault::ExecuteMsg,
    //     sender: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     router_ref.execute_contract(
    //         Addr::unchecked(sender),
    //         self.base.arb_vault.get_address(),
    //         &execute,
    //         &[],
    //     )
    // }

    // pub fn arb_fake_fill_arb_contract(&self, router_ref: &mut CustomApp) {
    //     let amount = 11000_000000u128;
    //     let result = self.hub_bond(router_ref, "fake", amount, "uluna").unwrap();

    //     let minted_event = result.events.iter().find(|f| f.ty == "wasm-erishub/bonded").unwrap();
    //     let minted_attribute =
    //         minted_event.attributes.iter().find(|a| a.key == "ustake_minted").unwrap();

    //     let amount = Uint128::from_str(&minted_attribute.value).unwrap();

    //     let ustake_addr = self.get_ustake_addr();
    //     let fake_addr = self.base.arb_fake_contract.get_address_string();

    //     // send minted ampTOKEN to the fake contract
    //     router_ref
    //         .execute_contract(
    //             Addr::unchecked("fake"),
    //             ustake_addr,
    //             &Cw20ExecuteMsg::Transfer {
    //                 recipient: fake_addr,
    //                 amount: amount,
    //             },
    //             &vec![],
    //         )
    //         .unwrap();
    // }

    // pub fn arb_stader_fake_fill_arb_contract(&self, router_ref: &mut CustomApp) {
    //     let amount = 11000_000000u128;
    //     let result = self.stader_bond(router_ref, "fake", amount, "uluna").unwrap();

    //     let minted_event = result.events.iter().find(|f| f.ty == "wasm").unwrap();
    //     let minted_attribute = minted_event.attributes.iter().find(|a| a.key == "amount").unwrap();

    //     let amount = Uint128::from_str(&minted_attribute.value).unwrap();

    //     let ustake_addr = self.get_stader_token_addr();
    //     let fake_addr = self.base.arb_fake_contract.get_address_string();

    //     // send minted ampTOKEN to the fake contract
    //     router_ref
    //         .execute_contract(
    //             Addr::unchecked("fake"),
    //             ustake_addr,
    //             &Cw20ExecuteMsg::Transfer {
    //                 recipient: fake_addr,
    //                 amount: amount,
    //             },
    //             &vec![],
    //         )
    //         .unwrap();
    // }

    // pub fn arb_steak_fake_fill_arb_contract(&self, router_ref: &mut CustomApp) {
    //     let amount = 11000_000000u128;
    //     let result = self.steak_bond(router_ref, "fake", amount, "uluna").unwrap();

    //     let minted_event = result.events.iter().find(|f| f.ty == "wasm-steakhub/bonded").unwrap();
    //     let minted_attribute =
    //         minted_event.attributes.iter().find(|a| a.key == "usteak_minted").unwrap();

    //     let amount = Uint128::from_str(&minted_attribute.value).unwrap();

    //     let ustake_addr = self.get_steak_token_addr();
    //     let fake_addr = self.base.arb_fake_contract.get_address_string();

    //     // send minted ampTOKEN to the fake contract
    //     router_ref
    //         .execute_contract(
    //             Addr::unchecked("fake"),
    //             ustake_addr,
    //             &Cw20ExecuteMsg::Transfer {
    //                 recipient: fake_addr,
    //                 amount: amount,
    //             },
    //             &vec![],
    //         )
    //         .unwrap();
    // }

    // pub fn arb_deposit(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     sender: &str,
    //     amount: u128,
    // ) -> Result<AppResponse> {
    //     router_ref.execute_contract(
    //         Addr::unchecked(sender),
    //         self.base.arb_vault.get_address(),
    //         &eris::arb_vault::ExecuteMsg::Deposit {
    //             asset: native_asset("uluna".to_string(), Uint128::new(amount)),
    //             receiver: None,
    //         },
    //         &[coin(amount, "uluna")],
    //     )
    // }

    // pub fn arb_withdraw(&self, router_ref: &mut CustomApp, sender: &str) -> Result<AppResponse> {
    //     self.arb_execute_sender(
    //         router_ref,
    //         eris::arb_vault::ExecuteMsg::WithdrawUnbonded {},
    //         sender,
    //     )
    // }

    // pub fn arb_remove_eris_lsd(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     self.arb_execute(
    //         router_ref,
    //         eris::arb_vault::ExecuteMsg::UpdateConfig {
    //             utilization_method: None,
    //             unbond_time_s: None,
    //             insert_lsd: None,
    //             disable_lsd: None,
    //             remove_lsd: Some("eris".to_string()),
    //             force_remove_lsd: None,
    //             fee_config: None,
    //             set_whitelist: None,
    //             remove_whitelist: None,
    //         },
    //     )
    // }

    // pub fn arb_stader_insert(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     self.arb_insert_lsd(
    //         router_ref,
    //         "stader".to_string(),
    //         LsdType::Stader {
    //             addr: self.base.stader.get_address_string(),
    //             cw20: self.base.stader_token.get_address_string(),
    //         },
    //     )
    // }

    // pub fn arb_steak_insert(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     self.arb_insert_lsd(
    //         router_ref,
    //         "boneLUNA".to_string(),
    //         LsdType::Backbone {
    //             addr: self.base.steak_hub.get_address_string(),
    //             cw20: self.base.steak_token.get_address_string(),
    //         },
    //     )
    // }

    // pub fn arb_insert_lsd(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     lsd: String,
    //     t: LsdType<String>,
    // ) -> Result<AppResponse> {
    //     self.arb_execute(
    //         router_ref,
    //         eris::arb_vault::ExecuteMsg::UpdateConfig {
    //             utilization_method: None,
    //             unbond_time_s: None,
    //             insert_lsd: Some(LsdConfig {
    //                 disabled: false,
    //                 name: lsd,
    //                 lsd_type: t,
    //             }),
    //             disable_lsd: None,
    //             remove_lsd: None,
    //             force_remove_lsd: None,
    //             fee_config: None,
    //             set_whitelist: None,
    //             remove_whitelist: None,
    //         },
    //     )
    // }

    // pub fn arb_unbond(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     sender: &str,
    //     amount: u128,
    //     immediate: Option<bool>,
    // ) -> Result<AppResponse> {
    //     let config = self.arb_query_config(router_ref).unwrap();
    //     let lp = config.config.lp_addr;

    //     let cw20msg = Cw20ExecuteMsg::Send {
    //         contract: self.base.arb_vault.get_address_string(),
    //         amount: Uint128::new(amount),
    //         msg: to_binary(&eris::arb_vault::Cw20HookMsg::Unbond {
    //             immediate,
    //         })
    //         .unwrap(),
    //     };

    //     router_ref.execute_contract(Addr::unchecked(sender), lp, &cw20msg, &[])
    // }

    // pub fn stader_bond(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     sender: impl Into<String>,
    //     amount: u128,
    //     denom: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     router_ref.execute_contract(
    //         Addr::unchecked(sender),
    //         self.base.stader.get_address(),
    //         &stader::msg::ExecuteMsg::Deposit {},
    //         &[coin(amount, denom.into())],
    //     )
    // }

    // pub fn stader_donate(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     sender: impl Into<String>,
    //     amount: u128,
    //     denom: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     router_ref
    //         .send_tokens(
    //             Addr::unchecked(sender),
    //             self.base.stader_reward.get_address(),
    //             &[coin(amount, denom.into())],
    //         )
    //         .unwrap();

    //     router_ref.execute_contract(
    //         self.owner.clone(),
    //         self.base.stader.get_address(),
    //         &stader::msg::ExecuteMsg::Reinvest {},
    //         &[],
    //     )
    // }

    // pub fn stader_harvest(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     router_ref
    //         .execute_contract(
    //             self.owner.clone(),
    //             self.base.stader.get_address(),
    //             &stader::msg::ExecuteMsg::RedeemRewards {
    //                 validators: None,
    //             },
    //             &[],
    //         )
    //         .unwrap_or_default();

    //     router_ref.execute_contract(
    //         self.owner.clone(),
    //         self.base.stader.get_address(),
    //         &stader::msg::ExecuteMsg::Reinvest {},
    //         &[],
    //     )
    // }

    // pub fn stader_submit_batch(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     router_ref.execute_contract(
    //         self.owner.clone(),
    //         self.base.stader.get_address(),
    //         &stader::msg::ExecuteMsg::Undelegate {},
    //         &[],
    //     )
    // }

    // pub fn stader_reconcile(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     amount: u128,
    // ) -> Result<AppResponse> {
    //     router_ref
    //         .sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
    //             to_address: self.base.stader.get_address_string(),
    //             amount: vec![coin(amount, "uluna")],
    //         }))
    //         .unwrap();

    //     router_ref.execute_contract(
    //         self.owner.clone(),
    //         self.base.stader.get_address(),
    //         &stader::msg::ExecuteMsg::ReconcileFunds {},
    //         &[],
    //     )
    // }

    // pub fn stader_query_state(
    //     &self,
    //     router_ref: &mut CustomApp,
    // ) -> StdResult<stader::msg::QueryStateResponse> {
    //     router_ref.wrap().query_wasm_smart(
    //         self.base.stader.get_address_string(),
    //         &stader::msg::QueryMsg::State {},
    //     )
    // }

    // pub fn stader_query_undelegation_records(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     user_addr: String,
    // ) -> StdResult<Vec<stader::state::UndelegationInfo>> {
    //     router_ref.wrap().query_wasm_smart(
    //         self.base.stader.get_address_string(),
    //         &stader::msg::QueryMsg::GetUserUndelegationRecords {
    //             user_addr,
    //             start_after: None,
    //             limit: None,
    //         },
    //     )
    // }

    // pub fn stader_query_batch_undelegation(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     batch_id: u64,
    // ) -> StdResult<stader::msg::QueryBatchUndelegationResponse> {
    //     router_ref.wrap().query_wasm_smart(
    //         self.base.stader.get_address_string(),
    //         &stader::msg::QueryMsg::BatchUndelegation {
    //             batch_id,
    //         },
    //     )
    // }

    // pub fn stader_query_delegations(&self, router_ref: &mut CustomApp) {
    //     let delegations =
    //         router_ref.wrap().query_all_delegations(self.base.stader.get_address_string()).unwrap();
    //     println!("delegations {:?}", delegations);

    //     let delegations =
    //         router_ref.wrap().query_all_balances(self.base.stader.get_address_string()).unwrap();
    //     println!("balances 1 {:?}", delegations);

    //     let delegations = router_ref
    //         .wrap()
    //         .query_all_balances(self.base.stader_reward.get_address_string())
    //         .unwrap();
    //     println!("balances 2 {:?}", delegations);
    // }

    // pub fn steak_execute(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     execute: steak::hub::ExecuteMsg,
    // ) -> Result<AppResponse> {
    //     router_ref.execute_contract(
    //         self.owner.clone(),
    //         self.base.steak_hub.get_address(),
    //         &execute,
    //         &[],
    //     )
    // }

    // pub fn steak_submit_batch(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     self.steak_execute(router_ref, steak::hub::ExecuteMsg::SubmitBatch {})
    // }

    // pub fn steak_reconcile(&self, router_ref: &mut CustomApp, amount: u128) -> Result<AppResponse> {
    //     router_ref
    //         .sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
    //             to_address: self.base.steak_hub.get_address_string(),
    //             amount: vec![coin(amount, "uluna")],
    //         }))
    //         .unwrap();

    //     self.steak_execute(router_ref, steak::hub::ExecuteMsg::Reconcile {})
    // }

    // pub fn steak_remove_validator(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     validator_addr: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     self.steak_execute(
    //         router_ref,
    //         steak::hub::ExecuteMsg::RemoveValidator {
    //             validator: validator_addr.into(),
    //         },
    //     )
    // }

    // pub fn steak_harvest(&self, router_ref: &mut CustomApp) -> Result<AppResponse> {
    //     self.steak_execute(router_ref, steak::hub::ExecuteMsg::Harvest {})
    // }

    // pub fn steak_add_validator(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     validator_addr: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     self.steak_execute(
    //         router_ref,
    //         steak::hub::ExecuteMsg::AddValidator {
    //             validator: validator_addr.into(),
    //         },
    //     )
    // }

    // pub fn steak_bond(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     sender: impl Into<String>,
    //     amount: u128,
    //     denom: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     router_ref.execute_contract(
    //         Addr::unchecked(sender),
    //         self.base.steak_hub.get_address(),
    //         &steak::hub::ExecuteMsg::Bond {
    //             receiver: None,
    //             exec_msg: None,
    //         },
    //         &[coin(amount, denom.into())],
    //     )
    // }

    // pub fn steak_donate(
    //     &self,
    //     router_ref: &mut CustomApp,
    //     sender: impl Into<String>,
    //     amount: u128,
    //     denom: impl Into<String>,
    // ) -> Result<AppResponse> {
    //     let sender_str: String = sender.into();
    //     let bonded = self.steak_bond(router_ref, sender_str.clone(), amount, denom).unwrap();

    //     let minted_event = bonded.events.iter().find(|f| f.ty == "wasm-steakhub/bonded").unwrap();
    //     let minted_attribute =
    //         minted_event.attributes.iter().find(|a| a.key == "usteak_minted").unwrap();
    //     let amount = Uint128::from_str(&minted_attribute.value).unwrap();

    //     router_ref.execute_contract(
    //         Addr::unchecked(sender_str),
    //         self.base.steak_token.get_address(),
    //         &cw20_base::msg::ExecuteMsg::Burn {
    //             amount,
    //         },
    //         &[],
    //     )
    // }
}
