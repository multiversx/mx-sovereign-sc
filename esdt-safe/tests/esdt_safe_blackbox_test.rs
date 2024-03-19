use multiversx_sc::
    types::{Address, ManagedByteArray}
;
use multiversx_sc_scenario::{
    api::StaticApi, scenario_model::{Account, AddressValue, SetStateStep}, ContractInfo, ScenarioWorld,
};

const ESDT_SAFE_PATH_EXPR: &str = "file:output/esdt_safe.wasm";
const INITIATOR_ADDRESS_EXPR: &str = "address:owner";
const SIGNERS_COUNT: u32 = 10;

type ESDTSafeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("esdt-safe/src/lib");

    blockchain.register_contract(ESDT_SAFE_PATH_EXPR, esdt_safe::ContractBuilder);

    blockchain
}

struct ESDTSafeTestState {
    world: ScenarioWorld,
    min_valid_signers: u32,
    initiator_address: Address,
    signers: [u8],
}

// impl ESDTSafeTestState {
//     fn new() -> Self {
//         let mut world = world();
//
//         world.set_state_step(
//             SetStateStep::new()
//                 .put_account(INITIATOR_ADDRESS_EXPR, Account::new().nonce(1))
//         );
//
//         let initiator_address = AddressValue::from(INITIATOR_ADDRESS_EXPR).to_address();
//         let min_valid_signers = SIGNERS_COUNT;
//         let signers = ManagedByteArray::new_from_bytes(&[u8, 32]);
//
//         Self {
//             world,
//             min_valid_signers,
//             initiator_address,
//             signers,
//         }
//     }
// }
