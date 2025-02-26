use cross_chain::deposit_unit_tests_setup::{
    CONTRACT_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, ONE_HUNDRED_MILLION, OWNER_ADDRESS,
    OWNER_BALANCE, TESTING_SC_ADDRESS, TEST_TOKEN_ONE, TEST_TOKEN_TWO, TOKEN_HANDLER_ADDRESS, USER,
};
use multiversx_sc::{
    require,
    types::{BigUint, EsdtLocalRole, ManagedAddress, TestSCAddress, TokenIdentifier},
};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, scenario_model::Log, ReturnsHandledOrError, ReturnsLogs,
    ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};
use operation::aliases::{OptionalValueTransferDataTuple, PaymentsVec};
use proxies::{
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    sov_enshrine_esdt_safe_proxy::SovEnshrineEsdtSafeProxy,
    testing_sc_proxy::TestingScProxy,
};
use sov_enshrine_esdt_safe::SovEnshrineEsdtSafe;

pub const SOV_ENSHRINE_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("output/sov-enshrine-esdt-safe.mxsc.json");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");
const TESTING_SC_CODE_PATH: MxscPath = MxscPath::new("../testing-sc/output/testing-sc.mxsc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        SOV_ENSHRINE_ESDT_SAFE_CODE_PATH,
        sov_enshrine_esdt_safe::ContractBuilder,
    );
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);

    blockchain
}

pub struct SovEnshrineEsdtSafeTestState {
    pub world: ScenarioWorld,
}

impl SovEnshrineEsdtSafeTestState {
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

    pub fn deploy_contract(&mut self, token_handler_address: TestSCAddress) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovEnshrineEsdtSafeProxy)
            .init(token_handler_address)
            .code(SOV_ENSHRINE_ESDT_SAFE_CODE_PATH)
            .new_address(CONTRACT_ADDRESS)
            .run();

        self
    }

    pub fn deploy_contract_with_roles(&mut self) -> &mut Self {
        self.world
            .account(CONTRACT_ADDRESS)
            .nonce(1)
            .code(SOV_ENSHRINE_ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS)
            .esdt_roles(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
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

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .whitebox(sov_enshrine_esdt_safe::contract_obj, |sc| {
                sc.init(TOKEN_HANDLER_ADDRESS.to_managed_address());
            });

        self
    }

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(SovEnshrineEsdtSafeProxy)
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
            .to(CONTRACT_ADDRESS)
            .typed(SovEnshrineEsdtSafeProxy)
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

    pub fn deposit_with(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        should_return_logs: bool,
    ) {
        let response_tuple = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(SovEnshrineEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        let (tx_response, logs) = response_tuple;

        if should_return_logs {
            for log in logs {
                assert!(!log.topics.is_empty());
            }
        }

        match tx_response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(expected_error_message, Some(error.message.as_str())),
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
            .to(CONTRACT_ADDRESS)
            .typed(SovEnshrineEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .run()
    }
}
