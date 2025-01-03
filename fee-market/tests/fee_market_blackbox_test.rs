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
use proxies::fee_market_proxy::{FeeMarketProxy, FeeStruct, FeeType};

const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("output/fee-market.mxsc.json");
const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");

const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER_ADDRESS: TestAddress = TestAddress::new("user");

const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TDK-123456");
const DIFFERENT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRONG-123456");
const ANOTHER_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ANOTHER-123456");
const WRONG_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRONG-TOKEN");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);

    blockchain
}

struct FeeMarketTestState {
    world: ScenarioWorld,
}

impl FeeMarketTestState {
    fn new() -> Self {
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

    fn deploy_fee_market(&mut self) -> &mut Self {
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
            .typed(FeeMarketProxy)
            .init(ESDT_SAFE_ADDRESS, Option::Some(fee))
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    fn substract_fee(&mut self, payment_wanted: &str, error_message: Option<&str>) {
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
            .typed(FeeMarketProxy)
            .subtract_fee(USER_ADDRESS, 1u8, OptionalValue::Some(30u64))
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn remove_fee(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .remove_fee(TOKEN_ID)
            .run();
    }

    fn add_fee(
        &mut self,
        token_id: TestTokenIdentifier,
        fee_type: &str,
        error_message: Option<&str>,
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
            .typed(FeeMarketProxy)
            .set_fee(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn add_users_to_whitelist(&mut self) {
        let mut users_vec = ManagedVec::new();
        users_vec.push(USER_ADDRESS.to_managed_address());
        let users = MultiValueEncoded::from(users_vec);
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .add_users_to_whitelist(users)
            .run();
    }

    fn check_balance_sc(&mut self, address: TestSCAddress, expected_balance: u64) {
        self.world
            .check_account(address)
            .esdt_balance(TOKEN_ID, expected_balance);
    }

    fn check_account(&mut self, address: TestAddress, expected_balance: u64) {
        self.world
            .check_account(address)
            .esdt_balance(TOKEN_ID, expected_balance);
    }
}

#[test]
fn test_deploy_fee_market() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();
}

#[test]
fn test_add_fee_wrong_params() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.add_fee(WRONG_TOKEN_ID, "Fixed", Some("Invalid token ID"));

    state.add_fee(TOKEN_ID, "None", Some("Invalid fee type"));

    state.add_fee(DIFFERENT_TOKEN_ID, "Fixed", Some("Invalid fee"));

    state.add_fee(TOKEN_ID, "AnyTokenWrong", Some("Invalid token ID"));
}

#[test]
fn test_substract_fee_no_fee() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();
    state.remove_fee();

    state.substract_fee("Correct", None);

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fee_whitelisted() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();
    state.add_users_to_whitelist();

    state.substract_fee("Correct", None);

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fee_invalid_payment_token() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.substract_fee("InvalidToken", Some("Token not accepted as fee"));

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

// FAILS => get_safe_price should be mocked here in order to make this test work
// #[test]
// fn test_substract_fee_any_token() {
//     let mut state = FeeMarketTestState::new();

//     state.deploy_fee_market();
//     state.add_fee(ANOTHER_TOKEN_ID, "AnyToken", None);

//     state.substract_fee("AnyToken", Some(ExpectError(4, "Invalid token provided for fee")));

// }

#[test]
fn test_substract_fixed_fee_payment_not_covered() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.substract_fee("Less than fee", Some("Payment does not cover fee"));

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 1000);
    state.check_account(USER_ADDRESS, 1000);
}

#[test]
fn test_substract_fee_fixed_payment_bigger_than_fee() {
    let mut state = FeeMarketTestState::new();

    state.deploy_fee_market();

    state.substract_fee("Correct", None);

    state.check_balance_sc(ESDT_SAFE_ADDRESS, 800);
    state.check_account(USER_ADDRESS, 1100);
}
