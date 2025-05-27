use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FEE_TOKEN, FIRST_TEST_TOKEN, HEADER_VERIFIER_ADDRESS,
    MVX_ESDT_SAFE_CODE_PATH, ONE_HUNDRED_MILLION, OWNER_ADDRESS, OWNER_BALANCE, SECOND_TEST_TOKEN,
    USER_ADDRESS,
};
use common_test_setup::{AccountSetup, BaseSetup, RegisterTokenArgs};
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtLocalRole, ManagedAddress, ManagedBuffer, ManagedVec, TestSCAddress,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun, ScenarioTxWhitebox,
};
use mvx_esdt_safe::{bridging_mechanism::TRUSTED_TOKEN_IDS, MvxEsdtSafe};
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;
use structs::{
    aliases::OptionalValueTransferDataTuple, configs::EsdtSafeConfig, operation::Operation,
};

pub struct MvxEsdtSafeTestState {
    pub common_setup: BaseSetup,
}

impl MvxEsdtSafeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_account = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![
                (FIRST_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into()),
                (SECOND_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into()),
                (FEE_TOKEN, 0u64, ONE_HUNDRED_MILLION.into()),
                (
                    TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
                    0u64,
                    ONE_HUNDRED_MILLION.into(),
                ),
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

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn deploy_contract_with_roles(&mut self) -> &mut Self {
        self.common_setup
            .world
            .account(ESDT_SAFE_ADDRESS)
            .nonce(1)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS)
            .esdt_roles(
                TokenIdentifier::from(FIRST_TEST_TOKEN),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                    EsdtLocalRole::Mint.name().to_string(),
                ],
            )
            .esdt_roles(
                TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                    EsdtLocalRole::Mint.name().to_string(),
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
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                let config = EsdtSafeConfig::new(
                    ManagedVec::new(),
                    ManagedVec::new(),
                    50_000_000,
                    ManagedVec::new(),
                    ManagedVec::new(),
                );

                sc.init(OptionalValue::Some(config));
            });

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn update_esdt_safe_config_during_setup_phase(
        &mut self,
        new_config: EsdtSafeConfig<StaticApi>,
        err_message: Option<&str>,
    ) {
        let result = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config_during_setup_phase(new_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, err_message);
    }

    pub fn update_esdt_safe_config(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        new_config: EsdtSafeConfig<StaticApi>,
        err_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (result, logs) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config(hash_of_hashes, new_config)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(result, err_message);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }

    pub fn set_token_burn_mechanism(
        &mut self,
        token_id: &str,
        expected_error_message: Option<&str>,
    ) -> &mut Self {
        let result = self
            .common_setup
            .world
            .tx()
            .from(HEADER_VERIFIER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .set_token_burn_mechanism(TokenIdentifier::from(token_id))
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);

        self
    }

    pub fn set_token_lock_mechanism(
        &mut self,
        token_id: &str,
        expected_error_message: Option<&str>,
    ) -> &mut Self {
        let result = self
            .common_setup
            .world
            .tx()
            .from(HEADER_VERIFIER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .set_token_lock_mechanism(TokenIdentifier::from(token_id))
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);

        self
    }

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .run();
    }

    pub fn deposit(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (logs, result) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data.clone())
            .payment(payment.clone())
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }

    pub fn register_token(
        &mut self,
        register_token_args: RegisterTokenArgs,
        payment: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let result = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .register_token(
                register_token_args.sov_token_id,
                register_token_args.token_type,
                ManagedBuffer::from(register_token_args.token_display_name),
                ManagedBuffer::from(register_token_args.token_ticker),
                register_token_args.num_decimals,
            )
            .egld(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);
    }

    pub fn register_native_token(
        &mut self,
        token_ticker: &str,
        token_name: &str,
        payment: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let result = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .register_native_token(
                ManagedBuffer::from(token_ticker),
                ManagedBuffer::from(token_name),
            )
            .egld(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);
    }

    pub fn execute_operation(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation: &Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
        expected_custom_log_data: Option<&str>,
    ) {
        let (logs, result) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);

        self.common_setup
            .assert_expected_log(logs.clone(), expected_custom_log);

        if let Some(custom_log_data) = expected_custom_log_data {
            self.common_setup
                .assert_expected_data(logs, custom_log_data);
        };
    }

    pub fn complete_setup_phase(
        &mut self,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (logs, result) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);

        self.common_setup
            .make_header_verifier_owner_of_the_sc(ESDT_SAFE_ADDRESS);
    }

    pub fn complete_setup_phase_as_header_verifier(
        &mut self,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (logs, result) = self
            .common_setup
            .world
            .tx()
            .from(HEADER_VERIFIER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }
}
