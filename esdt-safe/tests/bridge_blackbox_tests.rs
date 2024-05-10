use esdt_safe::esdt_safe_proxy::{self, EsdtSafeProxy};
use fee_market::fee_market_proxy;
use multiversx_sc::types::TestTokenIdentifier;
use multiversx_sc::{
    imports::{MultiValue3, MultiValueVec, OptionalValue},
    types::{
        BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, ReturnsResult, TestAddress,
        TestSCAddress,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ExpectError, ScenarioTxRun, ScenarioWorld,
};

const BRIDGE_ADDRESS: TestSCAddress = TestSCAddress::new("bridge");
const BRIDGE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const BRIDGE_OWNER_ADDRESS: TestAddress = TestAddress::new("bridge_owner");

const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee_market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price_aggregator");

const USER_ADDRESS: TestAddress = TestAddress::new("user");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const BRIDGE_OWNER_BALANCE: u64 = 100_000_000;
const USER_BALANCE: u64 = 100_000_000;

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");

const ESDT_PROXY: EsdtSafeProxy = esdt_safe_proxy::EsdtSafeProxy;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    // blockchain.set_current_dir_from_workspace("mx-sovereign-sc/esdt-safe");
    blockchain.register_contract(BRIDGE_CODE_PATH, esdt_safe::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);

    blockchain
}

struct BridgeTestState {
    world: ScenarioWorld,
}

impl BridgeTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(BRIDGE_OWNER_ADDRESS)
            .nonce(1)
            .balance(BRIDGE_OWNER_BALANCE)
            .account(USER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100, ManagedBuffer::new())
            .nonce(1)
            .balance(USER_BALANCE)
            .account(RECEIVER_ADDRESS)
            .nonce(1);

        Self { world }
    }

    fn deploy_bridge_contract(&mut self, is_sovereign_chain: bool) -> &mut Self {
        let signers = MultiValueVec::from(vec![USER_ADDRESS]);

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .typed(ESDT_PROXY)
            .init(is_sovereign_chain, 1u32, BRIDGE_OWNER_ADDRESS, signers)
            .code(BRIDGE_CODE_PATH)
            .new_address(BRIDGE_ADDRESS)
            .run();

        self.deploy_fee_market_contract();

        self.propose_set_unpaused();

        self
    }

    fn deploy_fee_market_contract(&mut self) -> &mut Self {
        let usdc_token_id = TestTokenIdentifier::new("USDC");
        let wegld_token_id = TestTokenIdentifier::new("WEGLD");

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .init(
                BRIDGE_ADDRESS,
                PRICE_AGGREGATOR_ADDRESS,
                usdc_token_id,
                wegld_token_id,
            )
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    fn propose_egld_deposit_and_expect_err(&mut self, err_message: &str) {
        let transfer_data = OptionalValue::<
            MultiValue3<
                u64,
                ManagedBuffer<StaticApi>,
                ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
            >,
        >::None;

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(ESDT_PROXY)
            .deposit(RECEIVER_ADDRESS, transfer_data)
            .egld(10)
            .with_result(ExpectError(4, err_message))
            .run();
    }

    fn propose_nft_deposit(&mut self) {
        let transfer_data = OptionalValue::<
            MultiValue3<
                u64,
                ManagedBuffer<StaticApi>,
                ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
            >,
        >::None;

        let payment = EsdtTokenPayment::new(NFT_TOKEN_ID.into(), 1, BigUint::from(10u64));

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(ESDT_PROXY)
            .deposit(RECEIVER_ADDRESS, transfer_data)
            .single_esdt(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            )
            .with_result(ReturnsResult)
            .run();
    }

    fn propose_set_unpaused(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(ESDT_PROXY)
            .unpause_endpoint()
            .returns(ReturnsResult)
            .run();
    }

    // fn propose_set_fee_market_address(&mut self) {
    //     self.world
    // }

    fn propose_execute_operations(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(ESDT_PROXY);
    }
}

#[test]
fn test_deploy() {
    let mut state = BridgeTestState::new();

    state.deploy_bridge_contract(false);
}

#[test]
fn test_egld_deposit_nothing_to_transfer() {
    let mut state = BridgeTestState::new();
    let err_message = "Nothing to transfer";

    state.deploy_bridge_contract(false);

    state.propose_egld_deposit_and_expect_err(err_message);
}

#[test]
fn test_esdt_deposit() {
    let mut state = BridgeTestState::new();

    state.deploy_bridge_contract(false);

    state.propose_nft_deposit();
}
