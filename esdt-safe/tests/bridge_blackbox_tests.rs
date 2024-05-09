use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        Address, ManagedAddress, ManagedBuffer, ManagedVec, MultiValueEncoded, TestAddress,
        TestSCAddress,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::MxscPath,
    managed_address, rust_biguint,
    scenario_model::{Account, AddressValue, ScCallStep, ScDeployStep, SetStateStep, TxExpect},
    ContractInfo, ScenarioWorld,
};
use esdt_safe::es
use transaction::GasLimit;

const BRIDGE_ADDRESS: TestSCAddress = TestSCAddress::new("bridge");
const BRIDGE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
// const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header_verifier");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER_ADDRESS: TestAddress = TestAddress::new("user");
const OWNER_BALANCE: u64 = 100_000_000;
const USER_BALANCE: u64 = 100_000_000;

type BridgeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("esdt-safe");
    blockchain.register_contract(BRIDGE_CODE_PATH, esdt_safe::ContractBuilder);

    blockchain
}

struct BridgeTestState {
    world: ScenarioWorld,
    // bridge_contract: BridgeContract,
    // is_sovereign_chain: bool,
    // min_valid_signers: u32,
    // initiator_address: Address,
    // signers: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>>,
}

impl BridgeTestState {
    fn new(is_sovereign_chain: bool) -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .balance(OWNER_BALANCE)
            .account(USER_ADDRESS)
            .nonce(1)
            .balance(USER_BALANCE);

        Self { world: world }
    }
    // fn new(is_sovereign_chain: bool) -> Self {
    //     let mut world = world();
    //
    //     let initiator_address = AddressValue::from(OWNER_ADDRESS_EXPR).to_address();
    //     let first_signer_account = AddressValue::from(FIRST_SINGER_ADDRESS_EXPR).to_address();
    //     let second_signer_account = AddressValue::from(SECOND_SINGER_ADDRESS_EXPR).to_address();
    //     let mut signers: MultiValueEncoded<_, multiversx_sc::types::ManagedAddress<_>> =
    //         MultiValueEncoded::new();
    //
    //     signers.push(managed_address!(&first_signer_account));
    //     signers.push(managed_address!(&second_signer_account));
    //
    //     world.set_state_step(
    //         SetStateStep::new()
    //             .put_account(
    //                 OWNER_ADDRESS_EXPR,
    //                 Account::new().nonce(1).balance(OWNER_BALANCE_EXPR),
    //             )
    //             .new_address(OWNER_ADDRESS_EXPR, 1, BRIDGE_ADDRESS_EXPR),
    //     );
    //
    //     let bridge_contract = BridgeContract::new(BRIDGE_ADDRESS_EXPR);
    //
    //     Self {
    //         world,
    //         bridge_contract,
    //         is_sovereign_chain,
    //         min_valid_signers: 2,
    //         initiator_address,
    //         signers,
    //     }
    // }

    fn deploy_bridge_contract(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(
            .init()
            .code(BRIDGE_CODE_PATH)
            .new_address(BRIDGE_ADDRESS);
        // self.world.sc_deploy(
        //     ScDeployStep::new()
        //         .from(OWNER_ADDRESS_EXPR)
        //         .code(bridge_code.clone())
        //         .call(self.bridge_contract.init(
        //             self.is_sovereign_chain,
        //             self.min_valid_signers,
        //             self.initiator_address.clone(),
        //             self.signers.clone(),
        //         ))
        //         .expect(TxExpect::ok()),
        // );

        // self.world.sc_call(
        //     ScCallStep::new()
        //         .from(OWNER_ADDRESS_EXPR)
        //         .call(self.bridge_contract.pause_endpoint())
        //         .expect(TxExpect::ok()),
        // );

        self
    }

    fn propose_deposit(
        &mut self,
        to: Address,
        opt_transfer_data: OptionalValue<
            MultiValue3<
                GasLimit,
                ManagedBuffer<StaticApi>,
                ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
            >,
        >,
    ) {
        // self.world.sc_call_get_result(
        //     ScCallStep::new()
        //         .from(OWNER_ADDRESS_EXPR)
        //         .egld_value(rust_biguint!(1))
        //         .call(self.bridge_contract.deposit(to, opt_transfer_data))
        //         .expect(TxExpect::ok()),
        // )
    }
}

#[test]
fn test_deploy() {
    let mut state = BridgeTestState::new(false);

    state.deploy_bridge_contract();
}

#[test]
fn test_deposit() {
    let mut state = BridgeTestState::new(false);
    state.deploy_bridge_contract();
    let receiver_address = AddressValue::from(RECEIVER_ADDRESS_EXPR).to_address();

    state.propose_deposit(receiver_address, OptionalValue::None);
}
