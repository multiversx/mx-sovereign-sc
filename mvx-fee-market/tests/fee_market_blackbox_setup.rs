use multiversx_sc::{
    imports::OptionalValue,
    types::{
        Address, BigUint, EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ManagedBuffer, ManagedVec,
        MultiValueEncoded, ReturnsHandledOrError, TestAddress, TestTokenIdentifier,
    },
};
use multiversx_sc_scenario::imports::*;

use common_test_setup::{
    base_setup::init::{AccountSetup, BaseSetup},
    constants::{
        CROWD_TOKEN_ID, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FIRST_TEST_TOKEN,
        HEADER_VERIFIER_ADDRESS, MVX_ESDT_SAFE_CODE_PATH, OWNER_ADDRESS, OWNER_BALANCE,
        SECOND_TEST_TOKEN, USER_ADDRESS,
    },
};
use proxies::mvx_fee_market_proxy::MvxFeeMarketProxy;
use structs::fee::{
    AddUsersToWhitelistOperation, DistributeFeesOperation, FeeStruct, FeeType, RemoveFeeOperation,
    RemoveUsersFromWhitelistOperation, SetFeeOperation,
};

pub struct FeeMarketTestState {
    pub common_setup: BaseSetup,
}

pub enum WantedFeeType {
    Correct,
    InvalidToken,
    LessThanFee,
    None,
    Fixed,
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
            base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            fee_type: FeeType::Fixed {
                token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
                per_transfer: BigUint::from(100u64),
                per_gas: BigUint::from(0u64),
            },
        }
    }

    pub fn subtract_fee(
        &mut self,
        payment_wanted: WantedFeeType,
        original_caller: Address,
        total_transfers: usize,
        opt_gas_limit: OptionalValue<u64>,
        expected_error_message: Option<&str>,
    ) {
        let payment: EsdtTokenPayment<StaticApi> = match payment_wanted {
            WantedFeeType::Correct => EsdtTokenPayment::new(
                FIRST_TEST_TOKEN.to_token_identifier(),
                0u64,
                BigUint::from(200u64),
            ),
            WantedFeeType::InvalidToken => EsdtTokenPayment::new(
                SECOND_TEST_TOKEN.to_token_identifier(),
                0u64,
                BigUint::from(10u64),
            ),
            WantedFeeType::LessThanFee => EsdtTokenPayment::new(
                FIRST_TEST_TOKEN.to_token_identifier(),
                0u64,
                BigUint::from(0u64),
            ),
            _ => panic!("Invalid payment wanted type"),
        };

        let response = self
            .common_setup
            .world
            .tx()
            .from(ESDT_SAFE_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .subtract_fee(original_caller, total_transfers, opt_gas_limit)
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn remove_fee_during_setup_phase(&mut self, base_token: TestTokenIdentifier) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .remove_fee_during_setup_phase(base_token)
            .run();
    }

    pub fn remove_fee(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        remove_fee_operation: RemoveFeeOperation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        let (response, logs) = self
            .common_setup
            .world
            .tx()
            .from(HEADER_VERIFIER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .remove_fee(hash_of_hashes, remove_fee_operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_log, expected_log_error);
    }

    pub fn set_fee(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        set_fee_operation: SetFeeOperation<StaticApi>,
        expected_custom_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        let (response, logs) = self
            .common_setup
            .world
            .tx()
            .from(HEADER_VERIFIER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .set_fee(hash_of_hashes, set_fee_operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(response, None);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log, expected_log_error);
    }

    pub fn set_fee_during_setup_phase(
        &mut self,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
        fee_type: WantedFeeType,
        expected_error_message: Option<&str>,
    ) {
        let fee_struct: FeeStruct<StaticApi> = match fee_type {
            WantedFeeType::None => {
                let fee_type = FeeType::None;
                FeeStruct {
                    base_token: token_id.clone(),
                    fee_type,
                }
            }
            WantedFeeType::Fixed => {
                let fee_type = FeeType::Fixed {
                    token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
                    per_transfer: BigUint::from(10u8),
                    per_gas: BigUint::from(10u8),
                };
                FeeStruct {
                    base_token: token_id,
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
            .typed(MvxFeeMarketProxy)
            .set_fee_during_setup_phase(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn distribute_fees(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation: DistributeFeesOperation<StaticApi>,
        expected_custom_log: Option<&str>,
        expected_error_log: Option<&str>,
    ) {
        let (response, logs) = self
            .common_setup
            .world
            .tx()
            .from(HEADER_VERIFIER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .distribute_fees(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(response, None);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log, expected_error_log);
    }

    pub fn add_users_to_whitelist_during_setup_phase(&mut self, users_vector: Vec<TestAddress>) {
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
            .typed(MvxFeeMarketProxy)
            .add_users_to_whitelist_during_setup_phase(users)
            .run();
    }

    pub fn add_users_to_whitelist(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation: AddUsersToWhitelistOperation<StaticApi>,
    ) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .add_users_to_whitelist(hash_of_hashes, operation)
            .run();
    }

    pub fn remove_users_from_whitelist(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation: RemoveUsersFromWhitelistOperation<StaticApi>,
    ) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .remove_users_from_whitelist(hash_of_hashes, operation)
            .run();
    }
}
