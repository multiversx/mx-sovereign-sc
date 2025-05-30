use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenPayment, ManagedVec, MultiValueEncoded, TestAddress, TestTokenIdentifier,
    },
};
use multiversx_sc_scenario::{api::StaticApi, ReturnsHandledOrError, ScenarioTxRun};

use common_test_setup::{
    constants::{
        CROWD_TOKEN_ID, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FIRST_TEST_TOKEN,
        MVX_ESDT_SAFE_CODE_PATH, OWNER_ADDRESS, OWNER_BALANCE, SECOND_TEST_TOKEN, USER_ADDRESS,
        WRONG_TOKEN_ID,
    },
    AccountSetup, BaseSetup,
};
use proxies::fee_market_proxy::FeeMarketProxy;
use structs::fee::{FeeStruct, FeeType};

pub struct FeeMarketTestState {
    pub common_setup: BaseSetup,
}

impl FeeMarketTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_account = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![(FIRST_TEST_TOKEN, 0, BigUint::from(OWNER_BALANCE))]),
            egld_balance: None,
        };

        let user_account = AccountSetup {
            address: USER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![(FIRST_TEST_TOKEN, 0, BigUint::from(OWNER_BALANCE))]),
            egld_balance: None,
        };

        let esdt_safe_address = AccountSetup {
            address: ESDT_SAFE_ADDRESS.to_address(),
            code_path: Some(MVX_ESDT_SAFE_CODE_PATH),
            esdt_balances: Some(vec![
                (FIRST_TEST_TOKEN, 0, BigUint::from(OWNER_BALANCE)),
                (SECOND_TEST_TOKEN, 0, BigUint::from(OWNER_BALANCE)),
                (CROWD_TOKEN_ID, 0, BigUint::from(OWNER_BALANCE)),
            ]),
            egld_balance: None,
        };

        let account_setups = vec![owner_account, user_account, esdt_safe_address];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn get_fee(&self) -> FeeStruct<StaticApi> {
        FeeStruct {
            base_token: FIRST_TEST_TOKEN.to_token_identifier(),
            fee_type: FeeType::Fixed {
                token: FIRST_TEST_TOKEN.to_token_identifier(),
                per_transfer: BigUint::from(100u64),
                per_gas: BigUint::from(0u64),
            },
        }
    }

    pub fn substract_fee(&mut self, payment_wanted: &str, expected_error_message: Option<&str>) {
        let payment: EsdtTokenPayment<StaticApi> = match payment_wanted {
            "Correct" => EsdtTokenPayment::new(
                FIRST_TEST_TOKEN.to_token_identifier(),
                0u64,
                BigUint::from(200u64),
            ),
            "InvalidToken" => EsdtTokenPayment::new(
                SECOND_TEST_TOKEN.to_token_identifier(),
                0u64,
                BigUint::from(10u64),
            ),
            "AnyToken" => EsdtTokenPayment::new(
                CROWD_TOKEN_ID.to_token_identifier(),
                0u64,
                BigUint::from(10u64),
            ),
            "Less than fee" => EsdtTokenPayment::new(
                FIRST_TEST_TOKEN.to_token_identifier(),
                0u64,
                BigUint::from(0u64),
            ),
            _ => {
                panic!("Invalid payment wanted");
            }
        };

        let response = self
            .common_setup
            .world
            .tx()
            .from(ESDT_SAFE_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .subtract_fee(USER_ADDRESS, 1u8, OptionalValue::Some(30u64))
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn remove_fee_during_setup_phase(&mut self) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .remove_fee_during_setup_phase(FIRST_TEST_TOKEN.to_token_identifier())
            .run();
    }

    pub fn set_fee_during_setup_phase(
        &mut self,
        token_id: TestTokenIdentifier,
        fee_type: &str,
        expected_error_message: Option<&str>,
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
                    token: FIRST_TEST_TOKEN.to_token_identifier(),
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
                    base_fee_token: SECOND_TEST_TOKEN.to_token_identifier(),
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
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .set_fee_during_setup_phase(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn add_users_to_whitelist(&mut self, users_vector: Vec<TestAddress>) {
        let mut users_vec = ManagedVec::new();

        for user in users_vector {
            users_vec.push(user.to_managed_address());
        }

        let users = MultiValueEncoded::from(users_vec);

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .add_users_to_whitelist(users)
            .run();
    }
}
