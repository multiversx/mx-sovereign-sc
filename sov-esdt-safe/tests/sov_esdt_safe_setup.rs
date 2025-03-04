use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EgldOrEsdtTokenIdentifier, EsdtLocalRole, ManagedAddress, ManagedVec, TestAddress,
        TestSCAddress, TestTokenIdentifier, TokenIdentifier,
    },
};

use multiversx_sc::contract_base::ContractBase;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, scenario_model::Log, ReturnsHandledOrError, ReturnsLogs,
    ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
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
use sov_esdt_safe::SovEsdtSafe;

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("sc");
pub const SOV_ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("output/to-sovereign.mxsc.json");

pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

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

    blockchain.register_contract(SOV_ESDT_SAFE_CODE_PATH, sov_esdt_safe::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
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
        fee_market_address: TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .init(fee_market_address, opt_config)
            .code(SOV_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self
    }

    pub fn update_configuration(&mut self, new_config: EsdtSafeConfig<StaticApi>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .update_configuration(new_config)
            .run();
    }

    pub fn deploy_contract_with_roles(&mut self) -> &mut Self {
        self.world
            .account(ESDT_SAFE_ADDRESS)
            .nonce(1)
            .code(SOV_ESDT_SAFE_CODE_PATH)
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
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(sov_esdt_safe::contract_obj, |sc| {
                let config = EsdtSafeConfig::new(
                    ManagedVec::new(),
                    ManagedVec::new(),
                    50_000_000,
                    ManagedVec::new(),
                );

                sc.init(
                    FEE_MARKET_ADDRESS.to_managed_address(),
                    OptionalValue::Some(config),
                );
            });

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
            .to(ESDT_SAFE_ADDRESS)
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

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.world
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
    ) -> Vec<Log> {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .run()
    }

    pub fn check_esdt_balance(&mut self, tokens: Vec<MultiValue3<TestTokenIdentifier, u64, u64>>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(sov_esdt_safe::contract_obj, |sc| {
                for token in tokens {
                    let (token_id, nonce, amount) = token.into_tuple();
                    let balance = sc
                        .blockchain()
                        .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), nonce);
                    assert_eq!(balance, amount);
                }
            });
    }
}
