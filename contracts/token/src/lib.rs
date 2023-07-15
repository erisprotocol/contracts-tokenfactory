use cosmwasm_std::{
    entry_point, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage,
};
use cw20_base::contract::{
    execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query,
};
use cw20_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cw20_base::state::{MinterData, TOKEN_INFO};
use cw20_base::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.mint.is_none() {
        return Err(StdError::generic_err("'mint' needs to be enabled").into());
    }

    cw20_instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // For `burn`, we assert that the caller is the minter
    // For `burn_from`, we simply disable it
    match msg {
        ExecuteMsg::Burn {
            ..
        } => assert_minter(deps.storage, &info.sender)?,
        ExecuteMsg::BurnFrom {
            ..
        } => return Err(StdError::generic_err("`burn_from` command is disabled").into()),
        _ => (),
    }

    cw20_execute(deps, env, info, msg)
}

fn assert_minter(storage: &dyn Storage, sender: &Addr) -> Result<(), ContractError> {
    let token_info = TOKEN_INFO.load(storage)?;

    if let Some(MinterData {
        minter,
        ..
    }) = &token_info.mint
    {
        if sender != minter {
            return Err(StdError::generic_err("only minter can execute token burn").into());
        }
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, env, msg)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{OwnedDeps, Uint128};
    use cw20_base::state::{TokenInfo, BALANCES};

    use super::*;

    fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();

        TOKEN_INFO
            .save(
                deps.as_mut().storage,
                &TokenInfo {
                    name: "Stake Token".to_string(),
                    symbol: "STAKE".to_string(),
                    decimals: 6,
                    total_supply: Uint128::new(200),
                    mint: Some(MinterData {
                        minter: Addr::unchecked("stake_hub"),
                        cap: None,
                    }),
                },
            )
            .unwrap();

        BALANCES
            .save(deps.as_mut().storage, &Addr::unchecked("stake_hub"), &Uint128::new(100))
            .unwrap();

        BALANCES
            .save(deps.as_mut().storage, &Addr::unchecked("alice"), &Uint128::new(100))
            .unwrap();

        deps
    }

    #[test]
    fn asserting_minter() {
        let mut deps = setup_test();

        // Alice is not allowed to burn her balance
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Burn {
                amount: Uint128::new(100),
            },
        );
        assert_eq!(res, Err(StdError::generic_err("only minter can execute token burn").into()));

        // Stake Hub can burn
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stake_hub", &[]),
            ExecuteMsg::Burn {
                amount: Uint128::new(100),
            },
        );
        assert!(res.is_ok());

        // Stake Hub's token balance should have been reduced
        let balance = BALANCES.load(deps.as_ref().storage, &Addr::unchecked("stake_hub")).unwrap();
        assert_eq!(balance, Uint128::zero());

        // Total supply should have been reduced
        let token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        assert_eq!(token_info.total_supply, Uint128::new(100));
    }

    #[test]
    fn disabling_burn_from() {
        let mut deps = setup_test();

        // Not even Stake Hub can invoke `burn_from`
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("stake_hub", &[]),
            ExecuteMsg::BurnFrom {
                owner: "alice".to_string(),
                amount: Uint128::new(100),
            },
        );
        assert_eq!(res, Err(StdError::generic_err("`burn_from` command is disabled").into()));
    }
}
