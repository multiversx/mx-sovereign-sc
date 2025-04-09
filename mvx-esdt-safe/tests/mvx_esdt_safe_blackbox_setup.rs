use common_blackbox_setup::{
    AccountSetup, BaseSetup, ESDT_SAFE_ADDRESS, FEE_TOKEN, HEADER_VERIFIER_ADDRESS,
    ONE_HUNDRED_MILLION, OWNER_ADDRESS, OWNER_BALANCE, TEST_TOKEN_ONE, TEST_TOKEN_TWO, USER,
};
use multiversx_sc::{
    codec::TopEncode,
    imports::OptionalValue,
    types::{
        BigUint, EsdtLocalRole, EsdtTokenType, ManagedAddress, ManagedBuffer, ManagedVec,
        MultiValueEncoded, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::Log, ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun, ScenarioTxWhitebox,
};
use mvx_esdt_safe::{bridging_mechanism::TRUSTED_TOKEN_IDS, MvxEsdtSafe};
use proxies::{header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy};
use structs::{
    aliases::OptionalValueTransferDataTuple, configs::EsdtSafeConfig, operation::Operation,
};

const CONTRACT_CODE_PATH: MxscPath = MxscPath::new("output/mvx-esdt-safe.mxsc.json");

pub struct RegisterTokenArgs<'a> {
    pub sov_token_id: TestTokenIdentifier<'a>,
    pub token_type: EsdtTokenType,
    pub token_display_name: &'a str,
    pub token_ticker: &'a str,
    pub num_decimals: usize,
}

pub struct MvxEsdtSafeTestState {
    pub common_setup: BaseSetup,
}

impl MvxEsdtSafeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_account = AccountSetup {
            address: OWNER_ADDRESS,
            esdt_balances: Some(vec![
                (
                    TestTokenIdentifier::new(TEST_TOKEN_ONE),
                    ONE_HUNDRED_MILLION.into(),
                ),
                (
                    TestTokenIdentifier::new(TEST_TOKEN_TWO),
                    ONE_HUNDRED_MILLION.into(),
                ),
                (
                    TestTokenIdentifier::new(FEE_TOKEN),
                    ONE_HUNDRED_MILLION.into(),
                ),
                (
                    TestTokenIdentifier::new(TRUSTED_TOKEN_IDS[0]),
                    ONE_HUNDRED_MILLION.into(),
                ),
            ]),
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let user_account = AccountSetup {
            address: USER,
            esdt_balances: Some(vec![(
                TestTokenIdentifier::new(TEST_TOKEN_ONE),
                ONE_HUNDRED_MILLION.into(),
            )]),
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let account_setups = vec![owner_account, user_account];

        let mut common_setup = BaseSetup::new(account_setups);

        common_setup
            .world
            .register_contract(CONTRACT_CODE_PATH, mvx_esdt_safe::ContractBuilder);

        Self { common_setup }
    }

    pub fn deploy_contract(
        &mut self,
        header_verifier_address: TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address, opt_config)
            .code(CONTRACT_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

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

    pub fn deploy_contract_with_roles(&mut self) -> &mut Self {
        self.common_setup
            .world
            .account(ESDT_SAFE_ADDRESS)
            .nonce(1)
            .code(CONTRACT_CODE_PATH)
            .owner(OWNER_ADDRESS)
            .esdt_roles(
                TokenIdentifier::from(TEST_TOKEN_ONE),
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
                TokenIdentifier::from(TEST_TOKEN_TWO),
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
                );

                sc.init(
                    HEADER_VERIFIER_ADDRESS.to_managed_address(),
                    OptionalValue::Some(config),
                );
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

    pub fn update_configuration(
        &mut self,
        new_config: EsdtSafeConfig<StaticApi>,
        err_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .update_configuration(new_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, err_message);
    }

    pub fn set_token_burn_mechanism(
        &mut self,
        token_id: &str,
        expected_error_message: Option<&str>,
    ) -> &mut Self {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .set_token_burn_mechanism(TokenIdentifier::from(token_id))
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }

        self
    }

    pub fn set_token_lock_mechanism(
        &mut self,
        token_id: &str,
        expected_error_message: Option<&str>,
    ) -> &mut Self {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .set_token_lock_mechanism(TokenIdentifier::from(token_id))
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }

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
        let (logs, response) = self
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
            .assert_expected_error_message(response, expected_error_message);

        if let Some(custom_log) = expected_custom_log {
            self.common_setup.assert_expected_log(logs, custom_log)
        };
    }

    pub fn register_token(
        &mut self,
        register_token_args: RegisterTokenArgs,
        payment: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
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
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn register_native_token(
        &mut self,
        token_ticker: &str,
        token_name: &str,
        payment: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
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
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn execute_operation(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation: &Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (logs, response) = self
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
            .assert_expected_error_message(response, expected_error_message);

        if let Some(custom_log) = expected_custom_log {
            self.common_setup.assert_expected_log(logs, custom_log)
        };
    }

    pub fn set_esdt_safe_address_in_header_verifier(&mut self, esdt_safe_address: TestSCAddress) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(esdt_safe_address)
            .run();
    }

    pub fn register_operation(
        &mut self,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                operations_hashes,
            )
            .run();
    }

    pub fn get_operation_hash(
        &mut self,
        operation: &Operation<StaticApi>,
    ) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }
}
