#[cfg(feature = "X-injective-X")]
pub mod injective;

#[cfg(feature = "X-osmosis-X")]
pub mod osmosis;

#[cfg(feature = "X-osmosis-X")]
pub mod types {
    pub type UsedCustomModule = super::osmosis::OsmosisModule;

    pub fn init_custom() -> UsedCustomModule {
        UsedCustomModule {}
    }
}

#[cfg(feature = "X-kujira-X")]
pub mod types {
    use self::kujira::KujiraModule;
    pub mod kujira;
    pub type UsedCustomModule = KujiraModule;

    pub fn init_custom() -> UsedCustomModule {
        UsedCustomModule {
            oracle_price: Decimal::zero(),
        }
    }
}

#[cfg(feature = "X-injective-X")]
pub mod types {
    pub type UsedCustomModule = super::injective::InjectiveModule;

    pub fn init_custom() -> UsedCustomModule {
        panic!("Cannot mock stargate messages in cw-multi-test")
    }
}
