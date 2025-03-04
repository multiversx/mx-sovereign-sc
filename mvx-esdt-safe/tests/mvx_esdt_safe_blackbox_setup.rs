use multiversx_sc::{
    codec::TopEncode,
    imports::{MultiValue2, OptionalValue},
    types::{
        BigUint, EsdtTokenType, ManagedAddress, ManagedBuffer, ManagedVec, MultiValueEncoded,
        TestAddress, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::Log, ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun, ScenarioWorld,
};
use operation::{
    aliases::OptionalValueTransferDataTuple, EsdtSafeConfig, Operation, SovereignConfig,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    testing_sc_proxy::TestingScProxy,
};

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("sc");
const CONTRACT_CODE_PATH: MxscPath = MxscPath::new("output/mvx-esdt-safe.mxsc.json");

pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

pub const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

pub const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
const CHAIN_CONFIG_CODE_PATH: MxscPath =
    MxscPath::new("../chain-config/output/chain-config.mxsc.json");

pub const TESTING_SC_ADDRESS: TestSCAddress = TestSCAddress::new("testing-sc");
const TESTING_SC_CODE_PATH: MxscPath = MxscPath::new("../testing-sc/output/testing-sc.mxsc.json");

pub const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const USER: TestAddress = TestAddress::new("user");

pub const TEST_TOKEN_ONE: &str = "TONE-123456";
pub const TEST_TOKEN_TWO: &str = "TTWO-123456";
pub const FEE_TOKEN: &str = "FEE-123456";

pub const ONE_HUNDRED_MILLION: u32 = 100_000_000;
pub const ONE_HUNDRED_THOUSAND: u32 = 100_000;
const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;

pub struct RegisterTokenArgs<'a> {
    pub sov_token_id: TestTokenIdentifier<'a>,
    pub token_type: EsdtTokenType,
    pub token_display_name: &'a str,
    pub token_ticker: &'a str,
    pub num_decimals: usize,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CONTRACT_CODE_PATH, mvx_esdt_safe::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);

    blockchain
}

pub struct MvxEsdtSafeTestState {
    pub world: ScenarioWorld,
}

impl MvxEsdtSafeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(
                TokenIdentifier::from(FEE_TOKEN),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_TWO),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .balance(BigUint::from(OWNER_BALANCE));

        world
            .account(USER)
            .nonce(1)
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .balance(BigUint::from(OWNER_BALANCE));

        Self { world }
    }

    pub fn deploy_contract(
        &mut self,
        header_verifier_address: TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address, opt_config)
            .code(CONTRACT_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self
    }

    pub fn update_configuration(
        &mut self,
        new_config: EsdtSafeConfig<StaticApi>,
        err_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .update_configuration(new_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                err_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(err_message, Some(error.message.as_str())),
        };
    }

    pub fn deploy_fee_market(&mut self, fee: Option<FeeStruct<StaticApi>>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(ESDT_SAFE_ADDRESS, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn deploy_testing_sc(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .new_address(TESTING_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_header_verifier(
        &mut self,
        bls_pub_keys: ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init(MultiValueEncoded::from(bls_pub_keys))
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    pub fn deploy_chain_config(&mut self, config: SovereignConfig<StaticApi>) -> &mut Self {
        let mut additional_stake_as_tuple = MultiValueEncoded::new();
        if let Some(additional_stake) = config.opt_additional_stake_required {
            for stake in additional_stake {
                additional_stake_as_tuple.push(MultiValue2::from((stake.token_id, stake.amount)));
            }
        }

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(
                config.min_validators as usize,
                config.max_validators as usize,
                config.min_stake,
                OWNER_ADDRESS,
                additional_stake_as_tuple,
            )
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .run();

        self
    }

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.world
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

        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }
    }

    pub fn deposit_with_logs(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
    ) -> Vec<Log> {
        self.world
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

        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }
    }

    pub fn execute_operation(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
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
    }

    pub fn set_esdt_safe_address_in_header_verifier(&mut self, esdt_safe_address: TestSCAddress) {
        self.world
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
        self.world
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
