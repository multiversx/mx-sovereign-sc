use bls_signature::BLS_SIGNATURE_LEN;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::{rust_biguint, testing_framework::{BlockchainStateWrapper, ContractObjWrapper}, DebugApi};

multiversx_sc::derive_imports!();

pub static FUNGIBLE_TOKEN_ID: &[u8] = b"FUNGTOKEN-123456";
pub static NFT_TOKEN_ID: &[u8] = b"NFT-123456";
pub const TOKEN_BALANCE: u64 = 1_000_000_000_000_000_000;
pub static DUMMY_SIG: [u8; BLS_SIGNATURE_LEN] = [0; BLS_SIGNATURE_LEN];
pub static FEE_TOKEN_ID: &[u8] = b"FEE-123456";

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

        b_mock.set_esdt_balance(&user, FUNGIBLE_TOKEN_ID, &rust_biguint!(TOKEN_BALANCE));
        b_mock.set_esdt_balance(&user, FEE_TOKEN_ID, &rust_biguint!(TOKEN_BALANCE));

        b_mock.set_nft_balance(
            &user,
            NFT_TOKEN_ID,
            1,
            &rust_biguint!(TOKEN_BALANCE),
            &DummyAttributes { dummy: 42 },
        );

        b_mock
            .execute_tx(&owner, &multisig_wrapper, &rust_zero, |sc| {
                sc.init();
            });

        MultiSigVerifierSetup { b_mock, owner, user, multisig_wrapper, bls_wrapper: todo!() }
    }
}
