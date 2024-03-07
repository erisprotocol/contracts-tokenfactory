use anyhow::Result;
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{coin, Addr, StdResult, Timestamp, Uint128};
use cw20::Logo;
use cw_multi_test::{App, AppBuilder, AppResponse, BankKeeper, ContractWrapper, Executor};
use eris::governance_helper::EPOCH_START;
use eris::voting_escrow::{
    BlacklistedVotersResponse, ExecuteMsg, InstantiateMsg, QueryMsg, UpdateMarketingInfo,
    VotingPowerResponse,
};

pub const MULTIPLIER: u64 = 1000000;

pub struct Helper {
    pub owner: Addr,
    pub stake: String,
    pub voting_instance: Addr,
}

#[allow(dead_code)]
impl Helper {
    pub fn init(router: &mut App, owner: Addr) -> Self {
        let voting_contract = Box::new(ContractWrapper::new_with_empty(
            eris_gov_voting_escrow::contract::execute,
            eris_gov_voting_escrow::contract::instantiate,
            eris_gov_voting_escrow::contract::query,
        ));

        let voting_code_id = router.store_code(voting_contract);

        let marketing_info = UpdateMarketingInfo {
            project: Some("Astroport".to_string()),
            description: Some("Astroport is a decentralized application for managing the supply of space resources.".to_string()),
            marketing: Some(owner.to_string()),
            logo: Some(Logo::Url("https://astroport.com/logo.png".to_string())),
        };

        let msg = InstantiateMsg {
            owner: owner.to_string(),
            guardian_addr: Some("guardian".to_string()),
            deposit_denom: "stake".to_string(),
            marketing: Some(marketing_info),
            logo_urls_whitelist: vec!["https://astroport.com/".to_string()],
        };
        let voting_instance = router
            .instantiate_contract(
                voting_code_id,
                owner.clone(),
                &msg,
                &[],
                String::from("vAMP"),
                None,
            )
            .unwrap();

        Self {
            owner,
            stake: "stake".to_string(),
            voting_instance,
        }
    }

    pub fn mint_xastro(&self, router: &mut App, to: &str, amount: u64) {
        let amount = amount * MULTIPLIER;

        router
            .sudo(cw_multi_test::SudoMsg::Bank(cw_multi_test::BankSudo::Mint {
                to_address: to.to_string(),
                amount: vec![coin(amount.into(), self.stake.to_string())],
            }))
            .unwrap();
    }

    pub fn check_xastro_balance(&self, router: &mut App, user: &str, amount: u64) {
        let amount = amount * MULTIPLIER;
        let res = router.wrap().query_balance(user.to_string(), self.stake.to_string()).unwrap();
        assert_eq!(res.amount.u128(), amount as u128);
    }

    // pub fn check_astro_balance(&self, router: &mut App, user: &str, amount: u64) {
    //     let amount = amount * MULTIPLIER;
    //     let res: BalanceResponse = router
    //         .wrap()
    //         .query_wasm_smart(
    //             self.astro_token.clone(),
    //             &Cw20QueryMsg::Balance {
    //                 address: user.to_string(),
    //             },
    //         )
    //         .unwrap();
    //     assert_eq!(res.balance.u128(), amount as u128);
    // }

    pub fn create_lock(
        &self,
        router: &mut App,
        user: &str,
        time: u64,
        amount: f32,
    ) -> Result<AppResponse> {
        let amount = (amount * MULTIPLIER as f32) as u64;
        router.execute_contract(
            Addr::unchecked(user),
            self.voting_instance.clone(),
            &ExecuteMsg::CreateLock {
                time,
            },
            &[coin(amount.into(), self.stake.to_string())],
        )
    }

    pub fn create_lock_u128(
        &self,
        router: &mut App,
        user: &str,
        time: u64,
        amount: u128,
    ) -> Result<AppResponse> {
        router.execute_contract(
            Addr::unchecked(user),
            self.voting_instance.clone(),
            &ExecuteMsg::CreateLock {
                time,
            },
            &[coin(amount, self.stake.clone())],
        )
    }

    pub fn extend_lock_amount(
        &self,
        router: &mut App,
        user: &str,
        amount: f32,
    ) -> Result<AppResponse> {
        self.extend_lock_amount_min(router, user, amount, None)
    }

    pub fn extend_lock_amount_min(
        &self,
        router: &mut App,
        user: &str,
        amount: f32,
        extend_to_min: Option<bool>,
    ) -> Result<AppResponse> {
        let amount = (amount * MULTIPLIER as f32) as u64;
        router.execute_contract(
            Addr::unchecked(user),
            self.voting_instance.clone(),
            &ExecuteMsg::ExtendLockAmount {
                extend_to_min_periods: extend_to_min,
            },
            &[coin(amount.into(), self.stake.clone())],
        )
    }

    pub fn deposit_for(
        &self,
        router: &mut App,
        from: &str,
        to: &str,
        amount: f32,
    ) -> Result<AppResponse> {
        let amount = (amount * MULTIPLIER as f32) as u64;
        router.execute_contract(
            Addr::unchecked(from),
            self.voting_instance.clone(),
            &ExecuteMsg::DepositFor {
                user: to.to_string(),
            },
            &[coin(amount.into(), self.stake.clone())],
        )
    }

    pub fn extend_lock_time(&self, router: &mut App, user: &str, time: u64) -> Result<AppResponse> {
        router.execute_contract(
            Addr::unchecked(user),
            self.voting_instance.clone(),
            &ExecuteMsg::ExtendLockTime {
                time,
            },
            &[],
        )
    }

    pub fn withdraw(&self, router: &mut App, user: &str) -> Result<AppResponse> {
        router.execute_contract(
            Addr::unchecked(user),
            self.voting_instance.clone(),
            &ExecuteMsg::Withdraw {},
            &[],
        )
    }

    pub fn update_blacklist(
        &self,
        router: &mut App,
        append_addrs: Option<Vec<String>>,
        remove_addrs: Option<Vec<String>>,
    ) -> Result<AppResponse> {
        router.execute_contract(
            Addr::unchecked("owner"),
            self.voting_instance.clone(),
            &ExecuteMsg::UpdateBlacklist {
                append_addrs,
                remove_addrs,
            },
            &[],
        )
    }

    pub fn update_decomission(
        &self,
        router: &mut App,
        sender: &str,
        decomission: Option<bool>
    ) -> Result<AppResponse> {
        router.execute_contract(
            Addr::unchecked(sender),
            self.voting_instance.clone(),
            &ExecuteMsg::UpdateConfig { new_guardian: None, push_update_contracts: None, decomission },
            &[],
        )
    }

    pub fn query_user_vp(&self, router: &mut App, user: &str) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::UserVamp {
                    user: user.to_string(),
                },
            )
            .map(|vp: VotingPowerResponse| vp.vamp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_exact_user_vp(&self, router: &mut App, user: &str) -> StdResult<u128> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::UserVamp {
                    user: user.to_string(),
                },
            )
            .map(|vp: VotingPowerResponse| vp.vamp.u128())
    }

    pub fn query_user_vp_at(&self, router: &mut App, user: &str, time: u64) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::UserVampAt {
                    user: user.to_string(),
                    time,
                },
            )
            .map(|vp: VotingPowerResponse| vp.vamp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_user_vp_at_period(
        &self,
        router: &mut App,
        user: &str,
        period: u64,
    ) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::UserVampAtPeriod {
                    user: user.to_string(),
                    period,
                },
            )
            .map(|vp: VotingPowerResponse| vp.vamp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_total_vp(&self, router: &mut App) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(self.voting_instance.clone(), &QueryMsg::TotalVamp {})
            .map(|vp: VotingPowerResponse| vp.vamp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_exact_total_vp(&self, router: &mut App) -> StdResult<u128> {
        router
            .wrap()
            .query_wasm_smart(self.voting_instance.clone(), &QueryMsg::TotalVamp {})
            .map(|vp: VotingPowerResponse| vp.vamp.u128())
    }

    pub fn query_total_vp_at(&self, router: &mut App, time: u64) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::TotalVampAt {
                    time,
                },
            )
            .map(|vp: VotingPowerResponse| vp.vamp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_total_vp_at_period(&self, router: &mut App, period: u64) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::TotalVampAtPeriod {
                    period,
                },
            )
            .map(|vp: VotingPowerResponse| vp.vamp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_locked_balance_at(
        &self,
        router: &mut App,
        user: &str,
        height: u64,
    ) -> StdResult<f32> {
        router
            .wrap()
            .query_wasm_smart(
                self.voting_instance.clone(),
                &QueryMsg::UserDepositAtHeight {
                    user: user.to_string(),
                    height,
                },
            )
            .map(|vp: Uint128| vp.u128() as f32 / MULTIPLIER as f32)
    }

    pub fn query_blacklisted_voters(
        &self,
        router: &mut App,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Vec<Addr>> {
        router.wrap().query_wasm_smart(
            self.voting_instance.clone(),
            &QueryMsg::BlacklistedVoters {
                start_after,
                limit,
            },
        )
    }

    pub fn check_voters_are_blacklisted(
        &self,
        router: &mut App,
        voters: Vec<String>,
    ) -> StdResult<BlacklistedVotersResponse> {
        router.wrap().query_wasm_smart(
            self.voting_instance.clone(),
            &QueryMsg::CheckVotersAreBlacklisted {
                voters,
            },
        )
    }
}

pub fn mock_app() -> App {
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(EPOCH_START);
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .build(|_, _, _| {})
}
