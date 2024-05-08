use esdt_safe::{to_sovereign::create_tx::ProxyTrait, ProxyTrait as _};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{Address, ManagedAddress, ManagedBuffer, ManagedVec, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    api::StaticApi,
    managed_address, rust_biguint,
    scenario_model::{Account, AddressValue, ScCallStep, ScDeployStep, SetStateStep, TxExpect},
    ContractInfo, ScenarioWorld,
};
use transaction::GasLimit;

const BRIDGE_PATH_EXPR: &str = "mxsc:output/esdt-safe.mxsc.json";
const BRIDGE_ADDRESS_EXPR: &str = "sc:bridge";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const OWNER_BALANCE_EXPR: &str = "100,000,000";
const RECEIVER_ADDRESS_EXPR: &str = "address:receiver";
const FIRST_SINGER_ADDRESS_EXPR: &str = "address:first_signer";
const SECOND_SINGER_ADDRESS_EXPR: &str = "address:second_signer";

type BridgeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("esdt-safe");
    blockchain.register_contract(BRIDGE_PATH_EXPR, esdt_safe::ContractBuilder);

    blockchain
}

struct BridgeTestState {
    world: ScenarioWorld,
    bridge_contract: BridgeContract,
    is_sovereign_chain: bool,
    min_valid_signers: u32,
    initiator_address: Address,
    signers: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>>,
}

impl BridgeTestState {
    fn new(is_sovereign_chain: bool) -> Self {
        let mut world = world();

        let initiator_address = AddressValue::from(OWNER_ADDRESS_EXPR).to_address();
        let first_signer_account = AddressValue::from(FIRST_SINGER_ADDRESS_EXPR).to_address();
        let second_signer_account = AddressValue::from(SECOND_SINGER_ADDRESS_EXPR).to_address();
        let mut signers: MultiValueEncoded<_, multiversx_sc::types::ManagedAddress<_>> =
            MultiValueEncoded::new();

        signers.push(managed_address!(&first_signer_account));
        signers.push(managed_address!(&second_signer_account));

        world.set_state_step(
            SetStateStep::new()
                .put_account(
                    OWNER_ADDRESS_EXPR,
                    Account::new().nonce(1).balance(OWNER_BALANCE_EXPR),
                )
                .new_address(OWNER_ADDRESS_EXPR, 1, BRIDGE_ADDRESS_EXPR),
        );

        let bridge_contract = BridgeContract::new(BRIDGE_ADDRESS_EXPR);

        Self {
            world,
            bridge_contract,
            is_sovereign_chain,
            min_valid_signers: 2,
            initiator_address,
            signers,
        }
    }

    fn deploy_bridge_contract(&mut self) -> &mut Self {
        let bridge_code = self.world.code_expression(BRIDGE_PATH_EXPR);

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(bridge_code.clone())
                .call(self.bridge_contract.init(
                    self.is_sovereign_chain,
                    self.min_valid_signers,
                    self.initiator_address.clone(),
                    self.signers.clone(),
                ))
                .expect(TxExpect::ok()),
        );

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
        self.world.sc_call_get_result(
            ScCallStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .egld_value(rust_biguint!(1))
                .call(self.bridge_contract.deposit(to, opt_transfer_data))
                .expect(TxExpect::ok()),
        )
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
