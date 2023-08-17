// pub mod asset;
pub mod adapters;
pub mod amp_gauges;
pub mod arb_vault;
pub mod astroport_farm;
pub mod compound_proxy;
pub mod constants;
pub mod emp_gauges;
pub mod fees_collector;
pub mod governance_helper;
pub mod helper;
pub mod helpers;
pub mod hub;
pub mod prop_gauges;
pub mod querier;
pub mod voting_escrow;

mod extensions {
    use crate::hub::CallbackMsg;
    use cosmwasm_std::{
        Attribute, Decimal, Decimal256, Env, Event, Fraction, OverflowError, Response, StdError,
        StdResult, Uint128, Uint256,
    };
    use eris_chain_adapter::types::CustomMsgType;
    use std::{convert::TryInto, str::FromStr};

    pub trait CustomEvent {
        fn add_optional_attribute(self, attribute: Option<Attribute>) -> Event;
    }

    impl CustomEvent for Event {
        fn add_optional_attribute(mut self, attribute: Option<Attribute>) -> Event {
            match attribute {
                Some(a) => {
                    self.attributes.push(a);
                    self
                },
                None => self,
            }
        }
    }

    pub trait CustomMsgExt {
        fn to_specific(self) -> StdResult<cosmwasm_std::CosmosMsg<CustomMsgType>>;
    }

    impl CustomMsgExt for cosmwasm_std::CosmosMsg {
        fn to_specific(self) -> StdResult<cosmwasm_std::CosmosMsg<CustomMsgType>> {
            match self {
                cosmwasm_std::CosmosMsg::Bank(msg) => Ok(cosmwasm_std::CosmosMsg::Bank(msg)),
                cosmwasm_std::CosmosMsg::Wasm(msg) => Ok(cosmwasm_std::CosmosMsg::Wasm(msg)),
                // cosmwasm_std::CosmosMsg::Staking(msg) => Ok(cosmwasm_std::CosmosMsg::Staking(msg)),
                // cosmwasm_std::CosmosMsg::Distribution(msg) => {
                //     Ok(cosmwasm_std::CosmosMsg::Distribution(msg))
                // },
                cosmwasm_std::CosmosMsg::Ibc(msg) => Ok(cosmwasm_std::CosmosMsg::Ibc(msg)),
                cosmwasm_std::CosmosMsg::Gov(msg) => Ok(cosmwasm_std::CosmosMsg::Gov(msg)),
                _ => Err(StdError::generic_err("not supported")),
            }
        }
    }

    pub trait CustomMsgExt2 {
        fn to_normal(self) -> StdResult<cosmwasm_std::CosmosMsg>;
    }

    impl CustomMsgExt2 for cosmwasm_std::CosmosMsg<CustomMsgType> {
        fn to_normal(self) -> StdResult<cosmwasm_std::CosmosMsg> {
            match self {
                cosmwasm_std::CosmosMsg::Bank(msg) => Ok(cosmwasm_std::CosmosMsg::Bank(msg)),
                cosmwasm_std::CosmosMsg::Wasm(msg) => Ok(cosmwasm_std::CosmosMsg::Wasm(msg)),
                // cosmwasm_std::CosmosMsg::Staking(msg) => Ok(cosmwasm_std::CosmosMsg::Staking(msg)),
                // cosmwasm_std::CosmosMsg::Distribution(msg) => {
                //     Ok(cosmwasm_std::CosmosMsg::Distribution(msg))
                // },
                cosmwasm_std::CosmosMsg::Ibc(msg) => Ok(cosmwasm_std::CosmosMsg::Ibc(msg)),
                cosmwasm_std::CosmosMsg::Gov(msg) => Ok(cosmwasm_std::CosmosMsg::Gov(msg)),
                _ => Err(StdError::generic_err("not supported")),
            }
        }
    }

    pub trait CustomResponse<T>: Sized {
        fn add_optional_message(self, msg: Option<cosmwasm_std::CosmosMsg<T>>) -> Self;
        fn add_optional_messages(self, msg: Option<Vec<cosmwasm_std::CosmosMsg<T>>>) -> Self;
        fn add_callback(self, env: &Env, msg: CallbackMsg) -> StdResult<Self>;
        fn add_optional_callback(self, env: &Env, msg: Option<CallbackMsg>) -> StdResult<Self>;
        fn add_optional_callbacks(
            self,
            env: &Env,
            msg: Option<Vec<CallbackMsg>>,
        ) -> StdResult<Self>;
    }

    impl CustomResponse<CustomMsgType> for Response<CustomMsgType> {
        fn add_optional_message(self, msg: Option<cosmwasm_std::CosmosMsg<CustomMsgType>>) -> Self {
            match msg {
                Some(msg) => self.add_message(msg),
                None => self,
            }
        }
        fn add_optional_messages(
            self,
            msg: Option<Vec<cosmwasm_std::CosmosMsg<CustomMsgType>>>,
        ) -> Self {
            match msg {
                Some(msgs) => self.add_messages(msgs),
                None => self,
            }
        }

        fn add_callback(self, env: &Env, msg: CallbackMsg) -> StdResult<Self> {
            Ok(self.add_message(msg.into_cosmos_msg(&env.contract.address)?))
        }

        fn add_optional_callback(self, env: &Env, msg: Option<CallbackMsg>) -> StdResult<Self> {
            match msg {
                Some(msg) => self.add_callback(env, msg),
                None => Ok(self),
            }
        }

        fn add_optional_callbacks(
            mut self,
            env: &Env,
            msg: Option<Vec<CallbackMsg>>,
        ) -> StdResult<Self> {
            match msg {
                Some(msgs) => {
                    for msg in msgs {
                        self = self.add_callback(env, msg)?;
                    }
                    Ok(self)
                },
                None => Ok(self),
            }
        }
    }
    pub trait DecimalCheckedOps {
        fn checked_add(self, other: Decimal) -> Result<Decimal, StdError>;
        fn checked_mul_uint(self, other: Uint128) -> Result<Uint128, StdError>;
        fn to_decimal256(self) -> Decimal256;
    }

    impl DecimalCheckedOps for Decimal {
        fn checked_add(self, other: Decimal) -> Result<Decimal, StdError> {
            self.numerator()
                .checked_add(other.numerator())
                .map(|_| self + other)
                .map_err(StdError::overflow)
        }

        fn checked_mul_uint(self, other: Uint128) -> Result<Uint128, StdError> {
            if self.is_zero() || other.is_zero() {
                return Ok(Uint128::zero());
            }
            let multiply_ratio =
                other.full_mul(self.numerator()) / Uint256::from(self.denominator());
            if multiply_ratio > Uint256::from(Uint128::MAX) {
                Err(StdError::overflow(OverflowError::new(
                    cosmwasm_std::OverflowOperation::Mul,
                    self,
                    other,
                )))
            } else {
                Ok(multiply_ratio.try_into().unwrap())
            }
        }

        fn to_decimal256(self) -> Decimal256 {
            Decimal256::from_str(&self.to_string()).unwrap()
        }
    }
}

pub use extensions::CustomEvent;
pub use extensions::CustomMsgExt;
pub use extensions::CustomMsgExt2;
pub use extensions::CustomResponse;
pub use extensions::DecimalCheckedOps;
