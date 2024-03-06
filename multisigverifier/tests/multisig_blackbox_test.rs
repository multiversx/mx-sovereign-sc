mod multisigverifier_setup;

use bls_signature::BlsSignature;
use multisigverifier::ProxyTrait;
use multiversx_sc::{
    codec::multi_types::MultiValueVec,
    types::{Address, ManagedBuffer, ManagedByteArray, ManagedVec},
};
use multiversx_sc_scenario::{
    api::StaticApi,
    scenario_model::{Account, AddressValue, ScCallStep, ScDeployStep, SetStateStep},
    ContractInfo, ScenarioWorld,
};

const MULTISIG_PATH_EXPR: &str = "file:output/multisigverifier.wasm";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const LEADER_ADDRESS_EXPR: &str = "address:proposer";
const VALIDATOR_ADDRESS_EXPR: &str = "address:board-member";
const MULTISIG_ADDRESS_EXPR: &str = "sc:multisig";

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
                .new_address(OWNER_ADDRESS_EXPR, 1, MULTISIG_ADDRESS_EXPR)
                .put_account(
                    LEADER_ADDRESS_EXPR,
                    Account::new().nonce(1).balance(LEADER_ADDRESS_EXPR),
                )
                .put_account(VALIDATOR_ADDRESS_EXPR, Account::new().nonce(1)),
        );

        let leader_address = AddressValue::from(LEADER_ADDRESS_EXPR).to_address();
        let validator_address = AddressValue::from(VALIDATOR_ADDRESS_EXPR).to_address();
        let multisig_contract = MultisigverifierContract::new(MULTISIG_ADDRESS_EXPR);

        Self {
            world,
            leader_address,
            validator_address,
            multisig_contract,
        }
    }

    fn deploy_multisig_contract(&mut self) -> &mut Self {
        let multisig_code = self.world.code_expression(MULTISIG_PATH_EXPR);
        let validators = MultiValueVec::from(vec![self.validator_address.clone()]);

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(multisig_code)
                .call(self.multisig_contract.init(validators)),
        );

        self
    }

    fn propose_register_bridge_ops(
        &mut self,
        bridge_operations_hash: &str,
        operations_hashes: ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
        signature: &bls_signature::BlsSignature<StaticApi>,
    ) {
        self.world
            .sc_call_get_result(ScCallStep::new().from(LEADER_ADDRESS_EXPR).call(
                self.multisig_contract.register_bridge_operations(
                    bridge_operations_hash,
                    operations_hashes,
                    signature,
                ),
            ))
    }
}

#[test]
fn test_deploy() {
    let mut state = MultisigTestState::new();
    state.deploy_multisig_contract();
}

#[test]
fn test_register_bridge_ops() {
    let mut state = MultisigTestState::new();
    state.deploy_multisig_contract();

    let bridge_operations_hash = "6ee1e00813a74f8293d2c63172c062d38bf780d8811ff63984813a49cd61ff9e";
    let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(
        b"EIZ2\x05\xf7q\xc7G\x96\x1f\xba0\xe2\xd1\xf5pE\x14\xd7?\xac\xff\x8d\x1a\x0c\x11\x900f5\xfb\xff4\x94\xb8@\xc5^\xc2,exn0\xe3\xf0\n"
    );

    let first_operation =
        ManagedBuffer::from("95cdb166d6e12a8c4a783a48d2e4f647e15fac4e5a115d4483f95881630a5433");
    let second_operation =
        ManagedBuffer::from("4851cd6e4a4799ad0d8e8ead37c88d930874302ab11edcc60f608654be14b2ed");

    let mut bridge_operations = ManagedVec::new();
    bridge_operations.push(first_operation);
    bridge_operations.push(second_operation);

    let _ = state.propose_register_bridge_ops(
        bridge_operations_hash,
        bridge_operations,
        &mock_signature,
    );
}
