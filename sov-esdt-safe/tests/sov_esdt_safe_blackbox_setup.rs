use multiversx_sc::{
    imports::OptionalValue,
    types::{
        EsdtLocalRole, ManagedAddress, ManagedVec, ReturnsHandledOrError, TestSCAddress,
        TokenIdentifier,
    },
};

use multiversx_sc_scenario::imports::*;

use common_test_setup::base_setup::init::{AccountSetup, BaseSetup};
use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, FIRST_TEST_TOKEN, ONE_HUNDRED_MILLION,
    OWNER_ADDRESS, OWNER_BALANCE, SECOND_TEST_TOKEN, SOV_ESDT_SAFE_CODE_PATH, USER_ADDRESS,
};
use proxies::sov_esdt_safe_proxy::SovEsdtSafeProxy;
use sov_esdt_safe::SovEsdtSafe;
use structs::{
    aliases::{OptionalValueTransferDataTuple, PaymentsVec},
    configs::EsdtSafeConfig,
    RegisterTokenStruct,
};

pub struct SovEsdtSafeTestState {
    pub common_setup: BaseSetup,
}

impl SovEsdtSafeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_account = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![
                (FIRST_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into()),
                (SECOND_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into()),
                (FEE_TOKEN, 0u64, ONE_HUNDRED_MILLION.into()),
            ]),
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let user_account = AccountSetup {
            address: USER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![(FIRST_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into())]),
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let account_setups = vec![owner_account, user_account];

        let mut common_setup = BaseSetup::new(account_setups);

        common_setup
            .world
            .register_contract(SOV_ESDT_SAFE_CODE_PATH, sov_esdt_safe::ContractBuilder);

        Self { common_setup }
    }

    pub fn deploy_contract_with_roles(&mut self) -> &mut Self {
        self.common_setup
            .world
            .account(ESDT_SAFE_ADDRESS)
            .nonce(1)
            .code(SOV_ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS)
            .esdt_roles(
                TokenIdentifier::from(FIRST_TEST_TOKEN),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                ],
            )
            .esdt_roles(
                TokenIdentifier::from(SECOND_TEST_TOKEN),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                ],
            )
            .esdt_roles(
                TokenIdentifier::from(FEE_TOKEN),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                ],
            );

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(sov_esdt_safe::contract_obj, |sc| {
                let config = EsdtSafeConfig::new(
                    ManagedVec::new(),
                    ManagedVec::new(),
                    50_000_000,
                    ManagedVec::new(),
                    ManagedVec::new(),
                );

                sc.init(
                    FEE_MARKET_ADDRESS.to_managed_address(),
                    OptionalValue::Some(config),
                );
            });

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deposit(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (logs, response) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .deposit(to, opt_transfer_data.clone())
            .payment(payment)
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_log, expected_error_message);
    }

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .run();
    }

    pub fn deposit_with_logs(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
        expected_log: Option<&str>,
    ) {
        let (logs, response) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, None);

        self.common_setup
            .assert_expected_log(logs, expected_log, None);
    }

    pub fn register_token(
        &mut self,
        new_token: RegisterTokenStruct<StaticApi>,
        payment: EgldOrEsdtTokenPayment<StaticApi>,
        expected_log: Option<&str>,
        expected_error_message: Option<&str>,
    ) {
        let (logs, response) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .register_token(
                new_token.token_id,
                new_token.token_type,
                new_token.token_display_name,
                new_token.token_ticker,
                new_token.num_decimals,
            )
            .payment(payment)
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_log, expected_error_message);
    }
}
