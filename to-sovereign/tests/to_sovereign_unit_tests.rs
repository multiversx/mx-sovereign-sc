use cross_chain::{storage::CrossChainStorage, DEFAULT_ISSUE_COST};
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc::{
    codec::TopEncode,
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedAddress, ManagedBuffer,
        ManagedVec, MultiValueEncoded, TestAddress, TestSCAddress, TestTokenIdentifier,
        TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::Log, ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};
use operation::{
    aliases::OptionalValueTransferDataTuple, EsdtSafeConfig, Operation, OperationData,
    OperationEsdtPayment, SovereignConfig, TransferData,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct, FeeType},
    header_verifier_proxy::HeaderverifierProxy,
    testing_sc_proxy::TestingScProxy,
    to_sovereign_proxy::ToSovereignProxy,
};

const CONTRACT_ADDRESS: TestSCAddress = TestSCAddress::new("sc");
const CONTRACT_CODE_PATH: MxscPath = MxscPath::new("output/to-sovereign.mxsc.json");

const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
const CHAIN_CONFIG_CODE_PATH: MxscPath =
    MxscPath::new("../chain-config/output/chain-config.mxsc.json");

const TESTING_SC_ADDRESS: TestSCAddress = TestSCAddress::new("testing-sc");
const TESTING_SC_CODE_PATH: MxscPath = MxscPath::new("../testing-sc/output/testing-sc.mxsc.json");

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER: TestAddress = TestAddress::new("user");

const TEST_TOKEN_ONE: &str = "TONE-123456";
const TEST_TOKEN_TWO: &str = "TTWO-123456";
const FEE_TOKEN: &str = "FEE-123456";

const ONE_HUNDRED_MILLION: u32 = 100_000_000;
const ONE_HUNDRED_THOUSAND: u32 = 100_000;
const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;

struct RegisterTokenArgs<'a> {
    sov_token_id: TestTokenIdentifier<'a>,
    token_type: EsdtTokenType,
    token_display_name: &'a str,
    token_ticker: &'a str,
    num_decimals: usize,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CONTRACT_CODE_PATH, to_sovereign::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);

    blockchain
}

struct ToSovereignTestState {
    world: ScenarioWorld,
}

impl ToSovereignTestState {
    fn new() -> Self {
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

    fn deploy_contract(
        &mut self,
        header_verifier_address: TestSCAddress,
        config: EsdtSafeConfig<StaticApi>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ToSovereignProxy)
            .init(header_verifier_address, config)
            .code(CONTRACT_CODE_PATH)
            .new_address(CONTRACT_ADDRESS)
            .run();

        self
    }

    fn deploy_fee_market(&mut self, fee: Option<FeeStruct<StaticApi>>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(CONTRACT_ADDRESS, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    fn deploy_testing_sc(&mut self) -> &mut Self {
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

    fn deploy_header_verifier(&mut self, chain_config_address: TestSCAddress) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init(chain_config_address)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    fn deploy_chain_config(&mut self, config: SovereignConfig<StaticApi>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(config, OWNER_ADDRESS)
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .run();

        self
    }

    fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
            .set_fee_market_address(fee_market_address)
            .run();
    }

    fn deposit(
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
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
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

    fn deposit_with_logs(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
    ) -> Vec<Log> {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .run()
    }

    fn register_token(
        &mut self,
        register_token_args: RegisterTokenArgs,
        payment: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
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

    fn execute_operation(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
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

    fn set_esdt_safe_address_in_header_verifier(&mut self, esdt_safe_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(esdt_safe_address)
            .run();
    }

    fn register_operation(
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

    fn complete_setup_phase(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .run();
    }

    fn change_validator_set(&mut self, validator_set: Vec<ManagedBuffer<StaticApi>>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .change_validator_set(MultiValueEncoded::from(ManagedVec::from(validator_set)))
            .run()
    }

    fn get_operation_hash(&mut self, operation: &Operation<StaticApi>) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }
}

#[test]
fn deploy() {
    let mut state = ToSovereignTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, EsdtSafeConfig::default_config());
}

#[test]
fn deposit_nothing_to_transfer() {
    let mut state = ToSovereignTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, EsdtSafeConfig::default_config());
    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        None,
        Some("Nothing to transfer"),
    );
}

#[test]
fn deposit_too_many_tokens() {
    let mut state = ToSovereignTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, EsdtSafeConfig::default_config());

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::default(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec),
        Some("Too many tokens"),
    );
}

#[test]
fn deposit_no_transfer_data() {
    let mut state = ToSovereignTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, EsdtSafeConfig::default_config());
    state.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec),
        None,
    );

    state
        .world
        .query()
        .to(CONTRACT_ADDRESS)
        .whitebox(to_sovereign::contract_obj, |sc| {
            assert!(sc
                .multiversx_to_sovereign_token_id_mapper(&TokenIdentifier::from(TEST_TOKEN_ONE))
                .is_empty());
        });
}

#[test]
fn deposit_gas_limit_too_high() {
    let mut state = ToSovereignTestState::new();

    let config = EsdtSafeConfig::new(ManagedVec::new(), ManagedVec::new(), 1, ManagedVec::new());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);
    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some("Gas limit too high"),
    );
}

#[test]
fn deposit_endpoint_banned() {
    let mut state = ToSovereignTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);
    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some("Banned endpoint name"),
    );
}

#[test]
fn deposit_fee_enabled() {
    let mut state = ToSovereignTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec.clone()),
        None,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        expected_amount_token_two,
    );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

#[test]
fn deposit_payment_doesnt_cover_fee() {
    let mut state = ToSovereignTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(TEST_TOKEN_ONE),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(TEST_TOKEN_ONE),
            per_transfer: BigUint::from(1u64),
            per_gas: BigUint::from(1u64),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some("Payment does not cover fee"),
    );
}

#[test]
fn deposit_refund() {
    let mut state = ToSovereignTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
    );

    for log in logs {
        assert!(!log.data.is_empty());
    }

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        &expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        &expected_amount_token_two,
    );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

#[test]
fn register_token_not_enough_egld() {
    let mut state = ToSovereignTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(1u64);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(
        register_token_args,
        egld_payment,
        Some("EGLD value should be 0.05"),
    );
}

#[test]
fn register_token_invalid_type() {
    let mut state = ToSovereignTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, Some("Invalid type"));
}

#[test]
fn register_token_fungible_token() {
    let mut state = ToSovereignTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = TEST_TOKEN_ONE;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // NOTE: Will use assert after framework fixes
    // state
    //     .world
    //     .query()
    //     .to(CONTRACT_ADDRESS)
    //     .whitebox(to_sovereign::contract_obj, |sc| {
    //         assert!(!sc
    //             .sovereign_to_multiversx_token_id_mapper(
    //                 &TestTokenIdentifier::new(TEST_TOKEN_ONE).into()
    //             )
    //             .is_empty());
    //     })
}

#[test]
fn register_token_nonfungible_token() {
    let mut state = ToSovereignTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "TokenOne";
    let num_decimals = 0;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // NOTE: Will use assert after framework fixes
    // state
    //     .world
    //     .query()
    //     .to(CONTRACT_ADDRESS)
    //     .whitebox(to_sovereign::contract_obj, |sc| {
    //         assert!(!sc
    //             .sovereign_to_multiversx_token_id_mapper(
    //                 &TestTokenIdentifier::new(TEST_TOKEN_ONE).into()
    //             )
    //             .is_empty());
    //     })
}

#[test]
fn execute_operation_no_esdt_safe_registered() {
    let mut state = ToSovereignTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = state.get_operation_hash(&operation);

    state.deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.execute_operation(
        hash_of_hashes,
        operation,
        Some("There is no registered ESDT address"),
    );
}

#[test]
fn execute_operation_success() {
    let mut state = ToSovereignTestState::new();
    let config = EsdtSafeConfig::default_config();
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(TEST_TOKEN_ONE), 0, token_data);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state.deploy_testing_sc();
    state.set_esdt_safe_address_in_header_verifier(CONTRACT_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.deploy_chain_config(SovereignConfig::default_config());
    state.change_validator_set(vec![ManagedBuffer::from("bls_1")]);
    state.complete_setup_phase();
    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let operation_hash_whitebox = ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
            let hash_of_hashes =
                ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

            assert!(
                sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .get()
                    == OperationHashStatus::NotLocked
            );
        });

    state.execute_operation(hash_of_hashes, operation.clone(), None);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let operation_hash_whitebox = ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
            let hash_of_hashes =
                ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

            assert!(sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                .is_empty());
        })
}
