use multiversx_sc::types::{BigUint, ManagedAddress, TestAddress, TestSCAddress, TokenIdentifier};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, scenario_model::Log, ReturnsHandledOrError, ReturnsLogs,
    ScenarioTxRun, ScenarioWorld,
};
use operation::{
    aliases::{OptionalValueTransferDataTuple, PaymentsVec},
    EsdtSafeConfig,
};
use proxies::{
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    sov_esdt_safe_proxy::SovEsdtSafeProxy,
    testing_sc_proxy::TestingScProxy,
};

pub const CONTRACT_ADDRESS: TestSCAddress = TestSCAddress::new("sc");
pub const CONTRACT_CODE_PATH: MxscPath = MxscPath::new("output/to-sovereign.mxsc.json");

pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

pub const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

// pub const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
// const CHAIN_CONFIG_CODE_PATH: MxscPath =
//     MxscPath::new("../chain-config/output/chain-config.mxsc.json");

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

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CONTRACT_CODE_PATH, sov_esdt_safe::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    // blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);

    blockchain
}

pub struct SovEsdtSafeTestState {
    pub world: ScenarioWorld,
}

impl SovEsdtSafeTestState {
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
        config: EsdtSafeConfig<StaticApi>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .init(header_verifier_address, config)
            .code(CONTRACT_CODE_PATH)
            .new_address(CONTRACT_ADDRESS)
            .run();

        self
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
            .to(CONTRACT_ADDRESS)
            .typed(SovEsdtSafeProxy)
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
            Err(error) => assert_eq!(expected_error_message, Some(error.message.as_str())),
        }
    }

    pub fn deploy_fee_market(&mut self, fee: Option<FeeStruct<StaticApi>>) -> &mut Self {
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

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .run();
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
            .to(CONTRACT_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .run()
    }
}
