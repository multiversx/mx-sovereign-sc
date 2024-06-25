#![allow(deprecated)]

use bls_signature::BlsSignature;
use header_verifier::ProxyTrait;
use multiversx_sc::types::{ManagedBuffer, ManagedByteArray, MultiValueEncoded};
use multiversx_sc_scenario::{
    api::StaticApi,
    scenario_model::{Account, ScCallStep, ScDeployStep, SetStateStep, TxExpect},
    ContractInfo, ScenarioWorld,
};

const HEADER_VERIFIER_PATH_EXPR: &str = "file:output/header-verifier.wasm";
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const LEADER_ADDRESS_EXPR: &str = "address:proposer";
const VALIDATOR_ADDRESS_EXPR: &str = "address:board-member";
const HEADER_VERIFIER_ADDRESS_EXPR: &str = "sc:header_verifier";

type HeaderVerifierContract = ContractInfo<header_verifier::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("header-verifier/src/lib");

    blockchain.register_contract(HEADER_VERIFIER_PATH_EXPR, header_verifier::ContractBuilder);

    blockchain
}

struct HeaderVerifierTestState {
    world: ScenarioWorld,
    header_verifier_contract: HeaderVerifierContract,
}

impl HeaderVerifierTestState {
    fn new() -> Self {
        let mut world = world();

        world.set_state_step(
            SetStateStep::new()
                .put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1))
                .new_address(OWNER_ADDRESS_EXPR, 1, HEADER_VERIFIER_ADDRESS_EXPR)
                .put_account(
                    LEADER_ADDRESS_EXPR,
                    Account::new().nonce(1).balance(LEADER_ADDRESS_EXPR),
                )
                .put_account(VALIDATOR_ADDRESS_EXPR, Account::new().nonce(1)),
        );

        let header_verifier_contract = HeaderVerifierContract::new(HEADER_VERIFIER_ADDRESS_EXPR);

        Self {
            world,
            header_verifier_contract,
        }
    }

    fn deploy_header_verifier_contract(&mut self) -> &mut Self {
        let header_verifier_code = self.world.code_expression(HEADER_VERIFIER_PATH_EXPR);

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(header_verifier_code),
        );

        self
    }

    fn _propose_register_bridge_ops(
        &mut self,
        bridge_operations_hash: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
        signature: &BlsSignature<StaticApi>,
    ) {
        self.world
            .sc_call_get_result(ScCallStep::new().from(LEADER_ADDRESS_EXPR).call(
                self.header_verifier_contract.register_bridge_operations(
                    signature,
                    bridge_operations_hash,
                    operations_hashes,
                ),
            ))
    }
}

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();
    state.deploy_header_verifier_contract();
}

#[test]
fn test_register_bridge_ops_wrong_hashes() {
    let mut state = HeaderVerifierTestState::new();
    state.deploy_header_verifier_contract();

    let bridge_operations_hash =
        ManagedBuffer::from("6ee1e00813a74f8293d2c63172c062d38bf780d8811ff63984813a49cd61ff9e");
    let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(
        b"EIZ2\x05\xf7q\xc7G\x96\x1f\xba0\xe2\xd1\xf5pE\x14\xd7?\xac\xff\x8d\x1a\x0c\x11\x900f5\xfb\xff4\x94\xb8@\xc5^\xc2,exn0\xe3\xf0\n"
    );

    let first_operation: ManagedBuffer<StaticApi> =
        ManagedBuffer::from("95cdb166d6e12a8c4a783a48d2e4f647e15fac4e5a115d4483f95881630a5433");
    let second_operation =
        ManagedBuffer::from("4851cd6e4a4799ad0d8e8ead37c88d930874302ab11edcc60f608654be14b2ed");

    let mut bridge_operations: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
        MultiValueEncoded::new();
    bridge_operations.push(first_operation);
    bridge_operations.push(second_operation);

    state.world.sc_call(
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .call(state.header_verifier_contract.register_bridge_operations(
                mock_signature,
                bridge_operations_hash,
                bridge_operations,
            ))
            .expect(TxExpect::user_error(
                "str:Hash of all operations doesn't match the hash of transfer data",
            )),
    );
}
