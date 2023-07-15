use cosmwasm_std::{OverflowError, StdError, Uint128};
use thiserror::Error;

/// ## Description
/// This enum describes pair contract errors!
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid message")]
    InvalidMessage {},

    #[error("Cannot unbond more than balance")]
    UnbondExceedBalance {},

    #[error("The config value for {0} is too high")]
    ConfigValueTooHigh(String),

    #[error(
        "Assertion failed; minimum receive amount: {minimum_receive}, actual amount: {amount}"
    )]
    AssertionMinimumReceive {
        minimum_receive: Uint128,
        amount: Uint128,
    },

    #[error("Invalid funds deposited")]
    InvalidFunds {},

    #[error("Expecting lp token, received {0}")]
    ExpectingLPToken(String),

    #[error("Expecting either amp lp for cw20 or amp lp denom for tokenfactory to be set.")]
    ExpectingAmpLpOrAmpLpDenom {},

    #[error("Use the cw20 unbond callback.")]
    ExpectingCw20Unbond {},

    #[error("Use the native unbond callback.")]
    ExpectingNativeUnbond {},
}

impl From<OverflowError> for ContractError {
    fn from(o: OverflowError) -> Self {
        StdError::from(o).into()
    }
}
