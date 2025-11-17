use crate::constants::*;
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{Address, BigUint, ManagedBuffer, MxscPath, TestTokenIdentifier, Vec},
    ScenarioWorld,
};
use std::borrow::Cow;
use structs::aliases::TxNonce;

pub struct BaseSetup {
    pub world: ScenarioWorld,
    operation_nonce: TxNonce,
}

pub struct AccountSetup<'a> {
    pub address: Address,
    pub code_path: Option<MxscPath<'a>>,
    pub esdt_balances: Option<Vec<(TestTokenIdentifier<'a>, u64, BigUint<StaticApi>)>>,
    pub egld_balance: Option<BigUint<StaticApi>>,
}

#[derive(Clone, Debug)]
pub struct ExpectedLogs<'a> {
    pub identifier: Cow<'a, str>,
    pub topics: OptionalValue<Vec<Cow<'a, str>>>,
    pub data: OptionalValue<Cow<'a, str>>,
}

pub trait ErrorPayloadToString {
    fn to_error_string(self) -> String;
}

impl ErrorPayloadToString for ManagedBuffer<StaticApi> {
    fn to_error_string(self) -> String {
        self.to_string()
    }
}

impl ErrorPayloadToString for Vec<u8> {
    fn to_error_string(self) -> String {
        ManagedBuffer::<StaticApi>::new_from_bytes(&self).to_string()
    }
}

#[macro_export]
macro_rules! log {
    ($identifier:expr) => {
        ExpectedLogs {
            identifier: ::std::borrow::Cow::<'_, str>::from($identifier),
            topics: OptionalValue::None,
            data: OptionalValue::None,
        }
    };
    ($identifier:expr, topics: [$($topic:expr),*]) => {
        ExpectedLogs {
            identifier: ::std::borrow::Cow::<'_, str>::from($identifier),
            topics: OptionalValue::Some(vec![$(::std::borrow::Cow::<'_, str>::from($topic)),*]),
            data: OptionalValue::None,
        }
    };
    ($identifier:expr, data: $data:expr) => {
        ExpectedLogs {
            identifier: ::std::borrow::Cow::<'_, str>::from($identifier),
            topics: OptionalValue::None,
            data: OptionalValue::Some(::std::borrow::Cow::<'_, str>::from($data)),
        }
    };
    ($identifier:expr, topics: [$($topic:expr),*], data: $data:expr) => {
        ExpectedLogs {
            identifier: ::std::borrow::Cow::<'_, str>::from($identifier),
            topics: OptionalValue::Some(vec![$(::std::borrow::Cow::<'_, str>::from($topic)),*]),
            data: match $data {
                Some(data) => OptionalValue::Some(::std::borrow::Cow::<'_, str>::from(data)),
                None => OptionalValue::None,
            },
        }
    };
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, mvx_fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);
    blockchain.register_contract(CHAIN_FACTORY_CODE_PATH, chain_factory::ContractBuilder);
    blockchain.register_contract(SOVEREIGN_FORGE_CODE_PATH, sovereign_forge::ContractBuilder);
    blockchain.register_contract(MVX_ESDT_SAFE_CODE_PATH, mvx_esdt_safe::ContractBuilder);
    blockchain.register_contract(SOV_FEE_MARKET_CODE_PATH, sov_fee_market::ContractBuilder);

    blockchain
}

impl BaseSetup {
    pub fn new(account_setups: Vec<AccountSetup>) -> Self {
        let mut world = world();

        for acc in account_setups {
            let mut acc_builder = match acc.code_path {
                Some(code_path) => world.account(acc.address.clone()).code(code_path).nonce(1),
                None => world.account(acc.address.clone()).nonce(1),
            };

            if let Some(esdt_balances) = &acc.esdt_balances {
                for (token_id, nonce, amount) in esdt_balances {
                    acc_builder = if *nonce != 0 {
                        acc_builder.esdt_nft_balance(
                            *token_id,
                            *nonce,
                            amount.clone(),
                            ManagedBuffer::new(),
                        )
                    } else {
                        acc_builder.esdt_balance(*token_id, amount.clone())
                    };
                }
            }

            if let Some(balance) = &acc.egld_balance {
                acc_builder.balance(balance.clone());
            }
        }

        Self {
            world,
            operation_nonce: 0,
        }
    }

    pub fn operation_nonce(&self) -> TxNonce {
        self.operation_nonce
    }

    pub fn next_operation_nonce(&mut self) -> TxNonce {
        let nonce = self.operation_nonce;
        self.operation_nonce = self
            .operation_nonce
            .checked_add(1)
            .expect("operation nonce overflow");
        nonce
    }
}
