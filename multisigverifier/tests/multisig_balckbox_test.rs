use bls_signature::BlsSignature;
use multisigverifier::ProxyTrait;
use multiversx_sc::{codec::multi_types::MultiValueVec, types::{Address, ManagedBuffer, ManagedByteArray, ManagedVec}};
use multiversx_sc_scenario::{api::StaticApi, scenario_model::{Account, AddressValue, ScCallStep, ScDeployStep, SetStateStep}, ContractInfo, ScenarioWorld};
use transaction::TransferData;

const MULTISIG_PATH_EXPR: &str = "file:output/multisigverifier.wasm";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const LEADER_ADDRESS_EXPR: &str = "address:proposer";
const VALIDATOR_ADDRESS_EXPR: &str = "address:board-member";

type MultisigverifierContract = ContractInfo<multisigverifier::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("multisigverifier/src/lib");

    blockchain.register_contract(MULTISIG_PATH_EXPR, multisigverifier::ContractBuilder);

    blockchain
}

struct MultisigTestState {
    world: ScenarioWorld,
    leader_address: Address,
    validator_address: Address,
    multisig_contract: MultisigverifierContract,
}

impl MultisigTestState {
    fn new() -> Self {
        let mut world = world();

        world.set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1))
                .new_address(OWNER_ADDRESS_EXPR, 1, MULTISIG_PATH_EXPR)
                .put_account(
                    LEADER_ADDRESS_EXPR, 
                    Account::new().nonce(1).balance(LEADER_ADDRESS_EXPR),
                )
                .put_account(VALIDATOR_ADDRESS_EXPR, Account::new().nonce(1))
        );

        let leader_address = AddressValue::from(LEADER_ADDRESS_EXPR).to_address();
        let validator_address = AddressValue::from(VALIDATOR_ADDRESS_EXPR).to_address(); 
        let multisig_contract = MultisigverifierContract::new(MULTISIG_PATH_EXPR);

        Self {
            world,
            leader_address,
            validator_address,
            multisig_contract
        }
    }

    fn deploy_multisig_contract(&mut self) -> &mut Self {
        let multisig_code = self.world.code_expression(MULTISIG_PATH_EXPR);
        let validators = MultiValueVec::from(vec![self.validator_address.clone()]);

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(multisig_code)
                .call(self.multisig_contract.init(validators))
        );

        self
    }

    fn propose_register_bridge_ops(
        &mut self,
        bridge_operations_hash: &str,
        signature: &bls_signature::BlsSignature<StaticApi>,
        bridge_operations: MultiValueVec<TransferData<StaticApi>>
    ) {
        self.world.sc_call_get_result(
            ScCallStep::new().from(LEADER_ADDRESS_EXPR).call(
                self.multisig_contract
                    .register_bridge_operations(bridge_operations_hash, signature, bridge_operations)
            )
        )
    }
}

#[test]
fn test_register_bridge_ops() {
    let mut state = MultisigTestState::new();
    state.deploy_multisig_contract();

    let bridge_operations_hash = "6ee1e00813a74f8293d2c63172c062d38bf780d8811ff63984813a49cd61ff9e";
    let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(
        b"EIZ2\x05\xf7q\xc7G\x96\x1f\xba0\xe2\xd1\xf5pE\x14\xd7?\xac\xff\x8d\x1a\x0c\x11\x900f5\xfb\xff4\x94\xb8@\xc5^\xc2,exn0\xe3\xf0\n"
    );

    let first_transfer_data = TransferData {
        args: ManagedVec::new(),
        gas_limit: 50000,
        function: ManagedBuffer::new()
    }; 

    let second_transfer_data = TransferData {
        args: ManagedVec::new(),
        gas_limit: 60000,
        function: ManagedBuffer::new()
    }; 

    let mut bridge_operations = MultiValueVec::new();
    bridge_operations.push(first_transfer_data);
    bridge_operations.push(second_transfer_data);

    let _ = state.propose_register_bridge_ops(bridge_operations_hash, &mock_signature, bridge_operations);
}
