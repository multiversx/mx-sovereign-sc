use common_blackbox_setup::{BaseSetup, ESDT_SAFE_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS};
use multiversx_sc::{
    codec::TopEncode,
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenType, ManagedAddress, ManagedBuffer, MultiValueEncoded, TestSCAddress,
        TestTokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::Log, ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun,
};
use proxies::{header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy};
use structs::{
    aliases::OptionalValueTransferDataTuple,
    configs::EsdtSafeConfig,
    operation::Operation,
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
        let mut common_setup = BaseSetup::new();

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
        opt_payment: Option<PaymentsVec<StaticApi>>,
        expected_error_message: Option<&str>,
    ) {
        let tx = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data);

        let response = if let Some(payment) = opt_payment {
            tx.payment(payment)
                .returns(ReturnsHandledOrError::new())
                .run()
        } else {
            tx.returns(ReturnsHandledOrError::new()).run()
        };

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn deposit_with_logs(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
    ) -> Vec<Log> {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .run()
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
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
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
            .register_bridge_operations(signature, hash_of_hashes, operations_hashes)
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
