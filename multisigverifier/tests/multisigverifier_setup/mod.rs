use multiversx_sc::types::Address;
use multiversx_sc_scenario::{rust_biguint, testing_framework::{BlockchainStateWrapper, ContractObjWrapper}, DebugApi};

multiversx_sc::derive_imports!();

pub struct MultiSigVerifierSetup<MultisigverifierBuilder, BlsMultisigBuilder>
where
    MultisigverifierBuilder: 'static + Copy + Fn() -> multisigverifier::ContractObj<DebugApi>,
    BlsMultisigBuilder: 'static + Copy + Fn() -> bls_signature::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner: Address,
    pub user: Address,
    pub multisig_wrapper: ContractObjWrapper<multisigverifier::ContractObj<DebugApi>, MultisigverifierBuilder>,
    pub bls_wrapper: ContractObjWrapper<bls_signature::ContractObj<DebugApi>, BlsMultisigBuilder>
}

impl<MultisigverifierBuilder, BlsMultisigBuilder> MultiSigVerifierSetup<MultisigverifierBuilder, BlsMultisigBuilder>
where
    MultisigverifierBuilder: 'static + Copy + Fn() -> multisigverifier::ContractObj<DebugApi>,
    BlsMultisigBuilder: 'static + Copy + Fn() -> bls_signature::ContractObj<DebugApi>,
{
    pub fn new(multisig_builder: MultisigverifierBuilder, bls_signature_builder: BlsMultisigBuilder) -> Self {
        let rust_zero = rust_biguint!(0);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner = b_mock.create_user_account(&rust_zero);
        let user = b_mock.create_user_account(&rust_zero);
        let multisig_wrapper = 
            b_mock.create_sc_account(&rust_zero, Some(&owner), multisig_builder, "multisig");
        let bls_wrappep = 
            b_mock.create_sc_account(&rust_zero, Some(&owner), multisig_builder, "bls");


        MultiSigVerifierSetup { b_mock, owner, user, multisig_wrapper, bls_wrapper: todo!() }
    }
}
