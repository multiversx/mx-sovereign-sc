use multiversx_sc::{
    imports::{FungibleTokenProperties, NonFungibleTokenProperties, SemiFungibleTokenProperties},
    types::{BigUint, EsdtTokenType},
};
use multiversx_sc_snippets::imports::StaticApi;

pub enum EsdtTokenProperties {
    Fungible(FungibleTokenProperties),
    NonFungible(NonFungibleTokenProperties),
    SemiFungible(SemiFungibleTokenProperties),
    None,
}

impl EsdtTokenProperties {
    pub fn new_fungible(props: Option<FungibleTokenProperties>) -> Self {
        EsdtTokenProperties::Fungible(props.unwrap_or_default())
    }

    pub fn new_non_fungible(props: Option<NonFungibleTokenProperties>) -> Self {
        EsdtTokenProperties::NonFungible(props.unwrap_or_default())
    }

    pub fn new_semi_fungible(props: Option<SemiFungibleTokenProperties>) -> Self {
        EsdtTokenProperties::SemiFungible(props.unwrap_or_default())
    }
}

pub enum IssueTokenStruct {
    Fungible {
        token_display_name: String,
        token_ticker: String,
        initial_supply: BigUint<StaticApi>,
        properties: EsdtTokenProperties,
    },
    NonFungible {
        token_display_name: String,
        token_ticker: String,
        properties: EsdtTokenProperties,
    },
    SemiFungible {
        token_display_name: String,
        token_ticker: String,
        properties: EsdtTokenProperties,
    },
    Dynamic {
        token_display_name: String,
        token_ticker: String,
        token_type: EsdtTokenType,
        num_decimals: usize,
    },
    Meta {
        token_display_name: String,
        token_ticker: String,
        num_decimals: usize,
    },
}
