use fee_market::fee_market_proxy::{self, FeeStruct, FeeType};

use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenPayment, ManagedVec, MultiValueEncoded, TestAddress, TestSCAddress,
        TestTokenIdentifier,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ReturnsHandledOrError, ScenarioTxRun, ScenarioWorld,
};

const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("output/fee-market.mxsc.json");
const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const USER_ADDRESS: TestAddress = TestAddress::new("user");

pub const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TDK-123456");
pub const DIFFERENT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRONG-123456");
const ANOTHER_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ANOTHER-123456");
pub const WRONG_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRONG-TOKEN");

pub fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);

    blockchain
}

pub struct FeeMarketTestState {
    pub world: ScenarioWorld,
}

impl FeeMarketTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .esdt_balance(TOKEN_ID, 1000)
            .nonce(1);

        world
            .account(USER_ADDRESS)
            .esdt_balance(TOKEN_ID, 1000)
            .nonce(1);

        world
            .account(ESDT_SAFE_ADDRESS)
            .code(ESDT_SAFE_CODE_PATH)
            .esdt_balance(TOKEN_ID, 1000)
            .esdt_balance(DIFFERENT_TOKEN_ID, 1000)
            .esdt_balance(ANOTHER_TOKEN_ID, 1000)
            .nonce(1);

        Self { world }
    }

    pub fn deploy_fee_market(&mut self) -> &mut Self {
        let fee = FeeStruct {
            base_token: TOKEN_ID.to_token_identifier(),
            fee_type: FeeType::Fixed {
                token: TOKEN_ID.to_token_identifier(),
                per_transfer: BigUint::from(100u64),
                per_gas: BigUint::from(0u64),
            },
        };

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .init(ESDT_SAFE_ADDRESS, Option::Some(fee))
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn substract_fee(&mut self, payment_wanted: &str, expected_result: Option<&str>) {
        let payment: EsdtTokenPayment<StaticApi> = match payment_wanted {
            "Correct" => {
                EsdtTokenPayment::new(TOKEN_ID.to_token_identifier(), 0u64, BigUint::from(200u64))
            }
            "InvalidToken" => EsdtTokenPayment::new(
                DIFFERENT_TOKEN_ID.to_token_identifier::<StaticApi>(),
                0u64,
                BigUint::from(10u64),
            ),
            "AnyToken" => EsdtTokenPayment::new(
                ANOTHER_TOKEN_ID.to_token_identifier(),
                0u64,
                BigUint::from(10u64),
            ),
            "Less than fee" => {
                EsdtTokenPayment::new(TOKEN_ID.to_token_identifier(), 0u64, BigUint::from(0u64))
            }
            _ => {
                panic!("Invalid payment wanted");
            }
        };

        let response = self
            .world
            .tx()
            .from(ESDT_SAFE_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .subtract_fee(USER_ADDRESS, 1u8, OptionalValue::Some(30u64))
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                expected_result.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(expected_result, Some(error.message.as_str())),
        };
    }

    pub fn remove_fee(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .remove_fee(TOKEN_ID)
            .run();
    }

    pub fn add_fee(
        &mut self,
        token_id: TestTokenIdentifier,
        fee_type: &str,
        expected_result: Option<&str>,
    ) {
        let fee_struct: FeeStruct<StaticApi> = match fee_type {
            "None" => {
                let fee_type = FeeType::None;
                FeeStruct {
                    base_token: token_id.to_token_identifier(),
                    fee_type,
                }
            }
            "Fixed" => {
                let fee_type = FeeType::Fixed {
                    token: TOKEN_ID.to_token_identifier(),
                    per_transfer: BigUint::from(10u8),
                    per_gas: BigUint::from(10u8),
                };
                FeeStruct {
                    base_token: token_id.to_token_identifier(),
                    fee_type,
                }
            }
            "AnyToken" => {
                let fee_type = FeeType::AnyToken {
                    base_fee_token: DIFFERENT_TOKEN_ID.to_token_identifier(),
                    per_transfer: BigUint::from(10u8),
                    per_gas: BigUint::from(10u8),
                };
                FeeStruct {
                    base_token: token_id.to_token_identifier(),
                    fee_type,
                }
            }
            "AnyTokenWrong" => {
                let fee_type = FeeType::AnyToken {
                    base_fee_token: WRONG_TOKEN_ID.to_token_identifier(),
                    per_transfer: BigUint::from(10u8),
                    per_gas: BigUint::from(10u8),
                };
                FeeStruct {
                    base_token: token_id.to_token_identifier(),
                    fee_type,
                }
            }
            _ => {
                panic!("Invalid fee type");
            }
        };

        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .set_fee(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                expected_result.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(expected_result, Some(error.message.as_str())),
        };
    }

    pub fn add_users_to_whitelist(&mut self) {
        let mut users_vec = ManagedVec::new();
        users_vec.push(USER_ADDRESS.to_managed_address());
        let users = MultiValueEncoded::from(users_vec);
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .add_users_to_whitelist(users)
            .run();
    }

    pub fn check_balance_sc(&mut self, address: TestSCAddress, expected_balance: u64) {
        self.world
            .check_account(address)
            .esdt_balance(TOKEN_ID, expected_balance);
    }

    pub fn check_account(&mut self, address: TestAddress, expected_balance: u64) {
        self.world
            .check_account(address)
            .esdt_balance(TOKEN_ID, expected_balance);
    }
}
