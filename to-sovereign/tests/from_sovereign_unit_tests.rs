use cross_chain::{storage::CrossChainStorage, DEFAULT_ISSUE_COST};
use multiversx_sc::{
    imports::{ESDTSystemSCProxy, OptionalValue},
    types::{
        BigUint, ESDTSystemSCAddress, EsdtLocalRole, EsdtTokenPayment, EsdtTokenType,
        ManagedAddress, ManagedBuffer, TestAddress, TestSCAddress, TestTokenIdentifier,
        TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ReturnsHandledOrError, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};
use operation::{aliases::OptionalValueTransferDataTuple, CrossChainConfig};
use proxies::{
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    to_sovereign_proxy::ToSovereignProxy,
};

const CONTRACT_ADDRESS: TestSCAddress = TestSCAddress::new("sc");
const CONTRACT_CODE_PATH: MxscPath = MxscPath::new("output/to-sovereign.mxsc.json");

const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER: TestAddress = TestAddress::new("user");

const TEST_TOKEN_ONE: &str = "test-token-one";
const TEST_TOKEN_TWO: &str = "test-token-two";

const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CONTRACT_CODE_PATH, to_sovereign::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);

    blockchain
}
struct FromSovereignTestState {
    world: ScenarioWorld,
}

impl FromSovereignTestState {
    fn new() -> Self {
        let mut world = world();
        // let roles = vec!["ESDTRoleNFTCreate".to_string(), "ESDTRoleBurn".to_string()];

        world
            .account(OWNER_ADDRESS)
            .nonce(3)
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                BigUint::from(100_000_000u64),
            )
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_TWO),
                BigUint::from(100_000_000u64),
            )
            .balance(BigUint::from(OWNER_BALANCE));

        Self { world }
    }

    fn deploy_contract(&mut self, config: CrossChainConfig<StaticApi>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ToSovereignProxy)
            .init(config)
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

    fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
            .set_fee_market_address(fee_market_address)
            .run();
    }

    fn register_token(
        &mut self,
        egld_amount: u64,
        sov_token_id: TestTokenIdentifier,
        token_type: EsdtTokenType,
        token_display_name: ManagedBuffer<StaticApi>,
        token_ticker: ManagedBuffer<StaticApi>,
        num_decimals: usize,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CONTRACT_ADDRESS)
            .typed(ToSovereignProxy)
            .register_token(
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn deposit(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        opt_payment: Option<PaymentsVec<StaticApi>>,
        error_message: Option<&str>,
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

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }
}

#[test]
fn deploy() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());
}

#[test]
fn register_token_not_enough_egld() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());

    let egld_amount = 5_000_000_000;
    let sov_token_id = TestTokenIdentifier::new("test-token");
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = ManagedBuffer::from("TTK");
    let token_ticker = ManagedBuffer::from("TTK");
    let num_decimals = 0 as usize;
    let error_message = Some("EGLD value should be 0.05");

    state.register_token(
        egld_amount,
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
        error_message,
    );
}

#[test]
fn register_token_fungible_token() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());

    let egld_amount = DEFAULT_ISSUE_COST;
    let sov_token_id = TestTokenIdentifier::new("test-token");
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = ManagedBuffer::from("TTK");
    let token_ticker = ManagedBuffer::from("TTK");
    let num_decimals = 0 as usize;

    state.register_token(
        egld_amount.into(),
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
        None,
    );

    state
        .world
        .query()
        .to(CONTRACT_ADDRESS)
        .whitebox(to_sovereign::contract_obj, |sc| {
            let sov_token_id_whitebox = &TestTokenIdentifier::new("test-token").into();

            assert!(!sc
                .sovereign_to_multiversx_token_id_mapper(sov_token_id_whitebox)
                .is_empty());

            let mvx_token_id = sc
                .sovereign_to_multiversx_token_id_mapper(sov_token_id_whitebox)
                .get();

            assert!(
                sc.multiversx_to_sovereign_token_id_mapper(&mvx_token_id)
                    .get()
                    == *sov_token_id_whitebox
            );

            assert!(
                sc.sovereign_to_multiversx_token_id_mapper(&sov_token_id_whitebox)
                    .get()
                    == mvx_token_id
            );
        })
}

#[test]
fn register_token_nonfungible_token() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());

    let egld_amount = DEFAULT_ISSUE_COST;
    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = ManagedBuffer::from("TTK");
    let token_ticker = ManagedBuffer::from("TTK");
    let num_decimals = 0 as usize;

    state.register_token(
        egld_amount.into(),
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
        None,
    );

    state
        .world
        .query()
        .to(CONTRACT_ADDRESS)
        .whitebox(to_sovereign::contract_obj, |sc| {
            let sov_token_id_whitebox = &TestTokenIdentifier::new(TEST_TOKEN_ONE).into();

            assert!(!sc
                .sovereign_to_multiversx_token_id_mapper(sov_token_id_whitebox)
                .is_empty());

            let mvx_token_id = sc
                .sovereign_to_multiversx_token_id_mapper(sov_token_id_whitebox)
                .get();

            assert!(
                sc.multiversx_to_sovereign_token_id_mapper(&mvx_token_id)
                    .get()
                    == *sov_token_id_whitebox
            );

            assert!(
                sc.sovereign_to_multiversx_token_id_mapper(&sov_token_id_whitebox)
                    .get()
                    == mvx_token_id
            );
        })
}

#[test]
fn deposit_nothing_to_transfer() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());
    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        None,
        Some("Nothing to transfer"),
    );
}

#[test]
fn deposit_too_many_tokens() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());

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
fn deposit() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());
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
        })
}
