use multiversx_sc::{
    imports::OptionalValue,
    types::{
        Address, BigUint, EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ReturnsHandledOrError,
        TestTokenIdentifier,
    },
};
use multiversx_sc_scenario::imports::*;

use common_test_setup::{
    base_setup::init::{AccountSetup, BaseSetup},
    constants::{
        CROWD_TOKEN_ID, ESDT_SAFE_ADDRESS, FIRST_TEST_TOKEN, MVX_ESDT_SAFE_CODE_PATH,
        OWNER_ADDRESS, OWNER_BALANCE, SECOND_TEST_TOKEN, SOV_FEE_MARKET_ADDRESS, USER_ADDRESS,
    },
};
use proxies::sov_fee_market_proxy::SovFeeMarketProxy;
use structs::fee::{FeeStruct, FeeType};

pub struct SovFeeMarketTestState {
    pub common_setup: BaseSetup,
}

pub enum WantedFeeType {
    Correct,
    InvalidToken,
    LessThanFee,
}

impl SovFeeMarketTestState {
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
        };

        let response = self
            .common_setup
            .world
            .tx()
            .from(ESDT_SAFE_ADDRESS)
            .to(SOV_FEE_MARKET_ADDRESS)
            .typed(SovFeeMarketProxy)
            .subtract_fee(original_caller, total_transfers, opt_gas_limit)
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn remove_fee(
        &mut self,
        token_id: TestTokenIdentifier,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOV_FEE_MARKET_ADDRESS)
            .typed(SovFeeMarketProxy)
            .remove_fee(token_id.to_token_identifier())
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn set_fee(
        &mut self,
        fee_struct: &FeeStruct<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOV_FEE_MARKET_ADDRESS)
            .typed(SovFeeMarketProxy)
            .set_fee(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn distribute_fees(
        &mut self,
        address_percentage_pairs: Vec<MultiValue2<ManagedAddress<StaticApi>, usize>>,
        expected_error_message: Option<&str>,
    ) {
        let mut pairs = MultiValueEncoded::new();
        for pair in address_percentage_pairs {
            pairs.push(pair);
        }
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOV_FEE_MARKET_ADDRESS)
            .typed(SovFeeMarketProxy)
            .distribute_fees(pairs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn add_users_to_whitelist(
        &mut self,
        users: Vec<ManagedAddress<StaticApi>>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOV_FEE_MARKET_ADDRESS)
            .typed(SovFeeMarketProxy)
            .add_users_to_whitelist(MultiValueEncoded::from(ManagedVec::from(users)))
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn remove_users_from_whitelist(
        &mut self,
        users: Vec<ManagedAddress<StaticApi>>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOV_FEE_MARKET_ADDRESS)
            .typed(SovFeeMarketProxy)
            .remove_users_from_whitelist(MultiValueEncoded::from(ManagedVec::from(users)))
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }
}
