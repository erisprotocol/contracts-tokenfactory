use cosmwasm_std::{Addr, Api, StdResult};

/// Returns a lowercased, validated address upon success if present.
pub fn addr_opt_validate(api: &dyn Api, addr: &Option<String>) -> StdResult<Option<Addr>> {
    addr.as_ref().map(|addr| api.addr_validate(addr)).transpose()
}
